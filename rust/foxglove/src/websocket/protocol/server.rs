use crate::channel::Channel;
use crate::channel::ChannelId;
use crate::FoxgloveError;
use base64::prelude::*;
use serde::Serialize;
use serde_json::json;
use serde_repr::Serialize_repr;
#[cfg(feature = "unstable")]
use serde_with::{base64::Base64, serde_as};
#[cfg(feature = "unstable")]
use std::collections::HashMap;
use std::collections::HashSet;

#[repr(u8)]
pub enum BinaryOpcode {
    MessageData = 1,
    // TimeData = 2,
    // ServiceCallResponse = 3,
    // FetchAssetResponse = 4,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Advertisement<'a> {
    pub id: ChannelId,
    pub topic: &'a str,
    pub encoding: &'a str,
    pub schema_name: &'a str,
    pub schema: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_encoding: Option<&'a str>,
}

/// A parameter type.
#[cfg(feature = "unstable")]
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    /// A byte array, encoded as a base64-encoded string.
    ByteArray,
    /// A decimal or integer value that can be represented as a `float64`.
    Float64,
    /// An array of decimal or integer values that can be represented as `float64`s.
    Float64Array,
}

/// A parameter value.
#[cfg(feature = "unstable")]
#[serde_as]
#[derive(Serialize)]
#[serde(untagged)]
pub enum ParameterValue {
    /// A decimal or integer value.
    Number(f64),
    /// A boolean value.
    Bool(bool),
    /// A byte array, encoded as a base64-encoded string.
    String(#[serde_as(as = "Base64")] Vec<u8>),
    /// An array of parameter values.
    Array(Vec<ParameterValue>),
    /// An associative map of parameter values.
    Dict(HashMap<String, ParameterValue>),
}

/// Informs the client about a parameter.
#[cfg(feature = "unstable")]
#[derive(Serialize)]
pub struct Parameter {
    /// The name of the parameter.
    pub name: String,
    /// The parameter value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<ParameterValue>,
    /// The parameter type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ParameterType>,
}

#[cfg(feature = "unstable")]
#[derive(Serialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
pub enum ServerMessage {
    ParameterValues {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        parameters: Vec<Parameter>,
    },
}

#[derive(Serialize_repr)]
#[repr(u8)]
pub enum StatusLevel {
    #[allow(dead_code)]
    Info = 0,
    #[allow(dead_code)]
    Warning = 1,
    Error = 2,
}

#[derive(Serialize)]
#[serde(tag = "op")]
#[serde(rename = "status")]
pub struct Status {
    pub level: StatusLevel,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// A capability that the websocket server advertises to its clients.
#[derive(Debug, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    /// Allow clients to advertise channels to send data messages to the server.
    ClientPublish,
    /// Allow clients to get & set parameters.
    #[cfg(feature = "unstable")]
    Parameters,
}

// https://github.com/foxglove/ws-protocol/blob/main/docs/spec.md#server-info
pub fn server_info(
    session_id: &str,
    name: &str,
    capabilities: &HashSet<Capability>,
    supported_encodings: &HashSet<String>,
) -> String {
    json!({
        "op": "serverInfo",
        "name": name,
        "capabilities": capabilities,
        "supportedEncodings": supported_encodings,
        "metadata": {},
        "sessionId": session_id
    })
    .to_string()
}

// https://github.com/foxglove/ws-protocol/blob/main/docs/spec.md#advertise
// Caller must check that the channel has a schema, otherwise this will panic.
pub fn advertisement(channel: &Channel) -> Result<String, FoxgloveError> {
    let schema = &channel
        .schema
        .as_ref()
        .ok_or_else(|| FoxgloveError::SchemaRequired)?;

    let schema_data = match schema.encoding.as_str() {
        "protobuf" => BASE64_STANDARD.encode(&schema.data),
        _ => String::from_utf8(schema.data.to_vec())
            .map_err(|e| FoxgloveError::Unspecified(e.into()))?,
    };

    Ok(json!({
        "op": "advertise",
        "channels": [Advertisement{
            id: channel.id,
            topic: &channel.topic,
            encoding: &channel.message_encoding,
            schema_name: &schema.name,
            schema: schema_data,
            schema_encoding: Some(&schema.encoding),
        }],
    })
    .to_string())
}

// https://github.com/foxglove/ws-protocol/blob/main/docs/spec.md#unadvertise
pub fn unadvertise(channel_id: ChannelId) -> String {
    json!({
        "op": "unadvertise",
        "channels": [channel_id],
    })
    .to_string()
}

#[cfg(feature = "unstable")]
pub fn parameters_json(
    parameters: Vec<Parameter>,
    id: Option<String>,
) -> Result<String, FoxgloveError> {
    serde_json::to_value(&ServerMessage::ParameterValues { parameters, id })
        .map(|value| value.to_string())
        .map_err(FoxgloveError::JSONError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info() {
        let default = server_info("id:123", "name:test", &HashSet::new(), &HashSet::new());
        let expected = json!({
            "op": "serverInfo",
            "name": "name:test",
            "sessionId": "id:123",
            "capabilities": [],
            "supportedEncodings": [],
            "metadata": {},
        });
        assert_eq!(default, expected.to_string());

        let with_publish = server_info(
            "id:123",
            "name:test",
            &HashSet::from([Capability::ClientPublish]),
            &HashSet::from(["json".to_string()]),
        );
        let expected = json!({
            "op": "serverInfo",
            "name": "name:test",
            "sessionId": "id:123",
            "capabilities": ["clientPublish"],
            "supportedEncodings": ["json"],
            "metadata": {},
        });
        assert_eq!(with_publish, expected.to_string());
    }

    #[test]
    fn test_status() {
        fn json(level: StatusLevel) -> serde_json::Value {
            let status = Status {
                level,
                message: "test".to_string(),
                id: None,
            };
            serde_json::to_value(&status).expect("Failed to serialize status")
        }

        let info_json = json(StatusLevel::Info);
        assert_eq!(
            info_json,
            json!({
                "op": "status",
                "level": 0,
                "message": "test",
            })
        );

        let warning_json = json(StatusLevel::Warning);
        assert_eq!(
            warning_json,
            json!({
                "op": "status",
                "level": 1,
                "message": "test",
            })
        );

        let error_json = json(StatusLevel::Error);
        assert_eq!(
            error_json,
            json!({
                "op": "status",
                "level": 2,
                "message": "test",
            })
        );
    }

    #[cfg(feature = "unstable")]
    #[test]
    fn test_parameter_values_byte_array() {
        let float_param = Parameter {
            name: "f64".to_string(),
            value: Some(ParameterValue::Number(1.23)),
            r#type: Some(ParameterType::Float64),
        };
        let float_array_param = Parameter {
            name: "f64[]".to_string(),
            value: Some(ParameterValue::Array(vec![
                ParameterValue::Number(1.23),
                ParameterValue::Number(4.56),
            ])),
            r#type: Some(ParameterType::Float64Array),
        };
        let data = vec![0x10, 0x20, 0x30];
        let byte_array_param = Parameter {
            name: "byte[]".to_string(),
            value: Some(ParameterValue::String(data.clone())),
            r#type: Some(ParameterType::ByteArray),
        };
        let bool_param = Parameter {
            name: "bool".to_string(),
            value: Some(ParameterValue::Bool(true)),
            r#type: None,
        };

        let parameters = vec![float_param, float_array_param, byte_array_param, bool_param];
        let result = parameters_json(parameters, None).unwrap();
        assert_eq!(
            result,
            json!({
                "op": "parameterValues",
                "parameters": [
                    {
                        "name": "f64",
                        "value": 1.23,
                        "type": "float64"
                    },
                    {
                        "name": "f64[]",
                        "value": [1.23, 4.56],
                        "type": "float64_array"
                    },
                    {
                        "name": "byte[]",
                        "value": BASE64_STANDARD.encode(data),
                        "type": "byte_array"
                    },
                    {
                        "name": "bool",
                        "value": true,
                    },
                ]
            })
            .to_string()
        );
    }

    #[cfg(feature = "unstable")]
    #[test]
    fn test_nested_named_parameter_values() {
        let inner_value = ParameterValue::Dict(HashMap::from([(
            "inner".to_string(),
            ParameterValue::Number(1.0),
        )]));
        let outer = Parameter {
            name: "outer".to_string(),
            value: Some(ParameterValue::Dict(HashMap::from([(
                "wrapping".to_string(),
                inner_value,
            )]))),
            r#type: None,
        };
        let parameters = vec![outer];
        let result = parameters_json(parameters, None).unwrap();
        assert_eq!(
            result,
            json!({
                "op": "parameterValues",
                "parameters": [
                    {
                        "name": "outer",
                        "value": {
                            "wrapping": {
                                "inner": 1.0
                            }
                        }
                    }
                ]
            })
            .to_string()
        );
    }

    #[cfg(feature = "unstable")]
    #[test]
    fn test_parameter_values_omitting_nulls() {
        let parameters = vec![Parameter {
            name: "test".to_string(),
            value: None,
            r#type: None,
        }];
        let result = parameters_json(parameters, None).unwrap();
        assert_eq!(
            result,
            json!({
                "op": "parameterValues",
                "parameters": [
                    {
                        "name": "test"
                    }
                ]
            })
            .to_string()
        );
    }
}
