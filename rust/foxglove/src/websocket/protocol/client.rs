//! Definitions of client-to-server messages in ws-protocol.
//! Serializations are derived for testing.

use crate::channel::ChannelId;
use serde::{Deserialize, Serialize};

type Error = Box<dyn std::error::Error>;

/// A client channel ID.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct ClientChannelId(u32);

impl ClientChannelId {
    /// Creates a new client channel ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl From<ClientChannelId> for u32 {
    fn from(id: ClientChannelId) -> u32 {
        id.0
    }
}

impl std::fmt::Display for ClientChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct SubscriptionId(u32);

impl SubscriptionId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl From<SubscriptionId> for u32 {
    fn from(id: SubscriptionId) -> u32 {
        id.0
    }
}

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[repr(u8)]
#[derive(strum::FromRepr)]
pub enum BinaryOpcode {
    MessageData = 1,
    ServiceCallRequest = 2,
}

// https://github.com/foxglove/ws-protocol/blob/main/docs/spec.md
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
pub enum ClientMessage {
    Subscribe {
        subscriptions: Vec<Subscription>,
    },
    Unsubscribe {
        subscription_ids: Vec<SubscriptionId>,
    },
    Advertise {
        channels: Vec<ClientChannel>,
    },
    Unadvertise {
        channel_ids: Vec<ClientChannelId>,
    },
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: SubscriptionId,
    pub channel_id: ChannelId,
}

#[doc(hidden)]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientChannel {
    pub id: ClientChannelId,
    pub topic: String,
    pub encoding: String,
    pub schema_name: String,
    pub schema_encoding: Option<String>,
    pub schema: Option<String>,
}

/// Parse message data from the client, returning the channel id and the payload:
/// [MCAP Client Message Data](https://github.com/foxglove/ws-protocol/blob/main/docs/spec.md#client-message-data)
#[doc(hidden)]
pub fn parse_binary_message(msg: &[u8]) -> Result<(ClientChannelId, &[u8]), Error> {
    // - 1-byte opcode == 0x01
    // - 4-byte channel id
    // - n-byte payload
    if msg.len() < 5 {
        return Err("message too short".into());
    }

    let channel_bytes = (&msg[1..=4]).try_into().unwrap();
    let channel_id = ClientChannelId::new(u32::from_le_bytes(channel_bytes));
    Ok((channel_id, &msg[5..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_advert_with_missiong_optional_fields() -> Result<(), serde_json::Error> {
        let msg = json!({
            "op": "advertise",
            "channels": [
                {
                    "id": 1,
                    "topic": "/test",
                    "encoding": "json",
                    "schemaName": "test",
                }
            ]
        });

        let result: Result<ClientMessage, _> = serde_json::from_str(&msg.to_string());
        assert_eq!(
            result?,
            ClientMessage::Advertise {
                channels: vec![ClientChannel {
                    id: ClientChannelId::new(1),
                    topic: "/test".to_string(),
                    encoding: "json".to_string(),
                    schema_name: "test".to_string(),
                    schema: None,
                    schema_encoding: None,
                }],
            },
        );
        Ok(())
    }

    #[test]
    fn test_unadvertise() -> Result<(), serde_json::Error> {
        let msg = json!({
            "op": "unadvertise",
            "channelIds": [1]
        });

        let result: Result<ClientMessage, _> = serde_json::from_str(&msg.to_string());
        assert_eq!(
            result?,
            ClientMessage::Unadvertise {
                channel_ids: vec![ClientChannelId::new(1)]
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_invalid_message() {
        let msg = vec![BinaryOpcode::MessageData as u8];
        let result = parse_binary_message(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsubscribe() {
        let msg = json!({
            "op": "unsubscribe",
            "subscriptionIds": [45]
        });

        let result: Result<ClientMessage, _> = serde_json::from_str(&msg.to_string());
        assert_eq!(
            result.expect("Failed to parse unsubscribe message"),
            ClientMessage::Unsubscribe {
                subscription_ids: vec![SubscriptionId::new(45)]
            }
        );
    }
}
