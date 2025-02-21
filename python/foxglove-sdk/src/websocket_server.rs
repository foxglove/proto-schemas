use crate::errors::PyFoxgloveError;
use foxglove::{
    websocket::{Client, ClientChannelView, ServerListener},
    WebSocketServer, WebSocketServerBlockingHandle,
};
use pyo3::{
    prelude::*,
    types::{PyBytes, PyString},
};
use std::time;
use std::{collections::HashMap, sync::Arc};

/// A client connected to a running websocket server.
#[pyclass(name = "Client", module = "foxglove")]
pub struct PyClient {
    #[pyo3(get)]
    id: u32,
}

/// Information about a client channel.
#[pyclass(name = "ClientChannelView", module = "foxglove")]
pub struct PyClientChannelView {
    #[pyo3(get)]
    id: u32,
    #[pyo3(get)]
    topic: Py<PyString>,
}

/// A mechanism to register callbacks for handling client message events.
///
/// Implementations of ServerListener which call the python methods. foxglove/__init__.py defines
/// the `ServerListener` protocol for callers, since a `pyclass` cannot extend Python classes:
/// https://github.com/PyO3/pyo3/issues/991
///
/// The ServerListener protocol implements all methods as no-ops by default; users extend this with
/// desired functionality.
///
/// Methods on the listener interface do not return Results; any errors are logged, assuming the
/// user has enabled logging.
pub struct PyServerListener {
    listener: Py<PyAny>,
}

impl ServerListener for PyServerListener {
    fn on_message_data(&self, client: Client, channel: ClientChannelView, payload: &[u8]) {
        let client_info = PyClient {
            id: client.id().into(),
        };

        let result: PyResult<()> = Python::with_gil(|py| {
            let channel_view = PyClientChannelView {
                id: channel.id().into(),
                topic: PyString::new(py, channel.topic()).into(),
            };

            // client, channel, data
            let args = (client_info, channel_view, PyBytes::new(py, payload));
            self.listener
                .bind(py)
                .call_method("on_message_data", args, None)?;

            Ok(())
        });

        if let Err(err) = result {
            tracing::error!("Callback failed: {}", err.to_string());
        }
    }

    fn on_get_parameters(
        &self,
        client: Client,
        param_names: Vec<String>,
        request_id: Option<&str>,
    ) -> Vec<foxglove::websocket::Parameter> {
        let client_info = PyClient {
            id: client.id().into(),
        };

        let result: PyResult<Vec<foxglove::websocket::Parameter>> = Python::with_gil(|py| {
            let args = (client_info, param_names, request_id);

            let result = self
                .listener
                .bind(py)
                .call_method("on_get_parameters", args, None)?;

            let parameters = result.extract::<Vec<PyParameter>>()?;

            Ok(parameters.into_iter().map(Into::into).collect())
        });

        match result {
            Ok(parameters) => parameters,
            Err(err) => {
                tracing::error!("Callback failed: {}", err.to_string());
                vec![]
            }
        }
    }

    fn on_set_parameters(
        &self,
        client: Client,
        parameters: Vec<foxglove::websocket::Parameter>,
        request_id: Option<&str>,
    ) -> Vec<foxglove::websocket::Parameter> {
        let client_info = PyClient {
            id: client.id().into(),
        };

        let result: PyResult<Vec<foxglove::websocket::Parameter>> = Python::with_gil(|py| {
            let parameters: Vec<PyParameter> = parameters.into_iter().map(Into::into).collect();
            let args = (client_info, parameters, request_id);

            let result = self
                .listener
                .bind(py)
                .call_method("on_set_parameters", args, None)?;

            let parameters = result.extract::<Vec<PyParameter>>()?;

            Ok(parameters.into_iter().map(Into::into).collect())
        });

        match result {
            Ok(parameters) => parameters,
            Err(err) => {
                tracing::error!("Callback failed: {}", err.to_string());
                vec![]
            }
        }
    }
}

/// Start a new Foxglove WebSocket server.
///
/// :param name: The name of the server.
/// :param host: The host to bind to.
/// :param port: The port to bind to.
/// :param capabilities: A list of capabilities to advertise to clients.
/// :param server_listener: A Python object that implements the :py:class:`ServerListener` protocol.
/// :param supported_encodings: A list of encodings to advertise to clients.
///    Foxglove currently supports "json", "ros1", and "cdr" for client-side publishing.
///
/// To connect to this server: open Foxglove, choose "Open a new connection", and select Foxglove
/// WebSocket. The default connection string matches the defaults used by the SDK.
#[pyfunction]
#[pyo3(signature = (*, name = None, host="127.0.0.1", port=8765, capabilities=None, server_listener=None, supported_encodings=None))]
pub fn start_server(
    py: Python<'_>,
    name: Option<String>,
    host: &str,
    port: u16,
    capabilities: Option<Vec<PyCapability>>,
    server_listener: Option<Py<PyAny>>,
    supported_encodings: Option<Vec<String>>,
) -> PyResult<PyWebSocketServer> {
    let session_id = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .expect("Failed to create session ID; invalid system time")
        .as_millis()
        .to_string();

    let mut server = WebSocketServer::new()
        .session_id(session_id)
        .bind(host, port);

    if let Some(py_obj) = server_listener {
        let listener = PyServerListener { listener: py_obj };
        server = server.listener(Arc::new(listener));
    }

    if let Some(name) = name {
        server = server.name(name);
    }

    if let Some(capabilities) = capabilities {
        server = server.capabilities(capabilities.into_iter().map(PyCapability::into));
    }

    if let Some(supported_encodings) = supported_encodings {
        server = server.supported_encodings(supported_encodings);
    }

    let handle = py
        .allow_threads(|| server.start_blocking())
        .map_err(PyFoxgloveError::from)?;

    Ok(PyWebSocketServer(Some(handle)))
}

/// A live visualization server. Obtain an instance by calling :py:func:`start_server`.
#[pyclass(name = "WebSocketServer", module = "foxglove")]
pub struct PyWebSocketServer(pub Option<WebSocketServerBlockingHandle>);

#[pymethods]
impl PyWebSocketServer {
    pub fn stop(&mut self, py: Python<'_>) {
        if let Some(server) = self.0.take() {
            py.allow_threads(|| server.stop())
        }
    }

    /// Sets a new session ID and notifies all clients, causing them to reset their state.
    /// If no session ID is provided, generates a new one based on the current timestamp.
    #[pyo3(signature = (session_id=None))]
    pub fn clear_session(&self, session_id: Option<String>) -> PyResult<()> {
        if let Some(server) = &self.0 {
            server.clear_session(session_id);
        }
        Ok(())
    }

    pub fn broadcast_time(&self, timestamp_nanos: u64) -> PyResult<()> {
        if let Some(server) = &self.0 {
            server.broadcast_time(timestamp_nanos);
        }
        Ok(())
    }

    /// Publishes parameter values to all clients.
    pub fn publish_parameter_values(&self, parameters: Vec<PyParameter>) {
        if let Some(server) = &self.0 {
            server.publish_parameter_values(parameters.into_iter().map(Into::into).collect());
        }
    }
}

/// A capability that the websocket server advertises to its clients.
#[pyclass(eq, eq_int, name = "Capability")]
#[derive(Clone, PartialEq)]
pub enum PyCapability {
    /// Allow clients to advertise channels to send data messages to the server.
    ClientPublish,
    /// Allow clients to get & set parameters.
    Parameters,
    /// Inform clients about the latest server time.
    ///
    /// This allows accelerated, slowed, or stepped control over the progress of time. If the
    /// server publishes time data, then timestamps of published messages must originate from the
    /// same time source.
    Time,
}

impl From<PyCapability> for foxglove::websocket::Capability {
    fn from(value: PyCapability) -> Self {
        match value {
            PyCapability::ClientPublish => foxglove::websocket::Capability::ClientPublish,
            PyCapability::Parameters => foxglove::websocket::Capability::Parameters,
            PyCapability::Time => foxglove::websocket::Capability::Time,
        }
    }
}

#[pyclass(name = "ParameterType", module = "foxglove", eq, eq_int)]
#[derive(Clone, PartialEq)]
pub enum PyParameterType {
    /// A byte array, encoded as a base64-encoded string.
    ByteArray,
    /// A decimal or integer value that can be represented as a `float64`.
    Float64,
    /// An array of decimal or integer values that can be represented as `float64`s.
    Float64Array,
}

impl From<PyParameterType> for foxglove::websocket::ParameterType {
    fn from(value: PyParameterType) -> Self {
        match value {
            PyParameterType::ByteArray => foxglove::websocket::ParameterType::ByteArray,
            PyParameterType::Float64 => foxglove::websocket::ParameterType::Float64,
            PyParameterType::Float64Array => foxglove::websocket::ParameterType::Float64Array,
        }
    }
}

impl From<foxglove::websocket::ParameterType> for PyParameterType {
    fn from(value: foxglove::websocket::ParameterType) -> Self {
        match value {
            foxglove::websocket::ParameterType::ByteArray => PyParameterType::ByteArray,
            foxglove::websocket::ParameterType::Float64 => PyParameterType::Float64,
            foxglove::websocket::ParameterType::Float64Array => PyParameterType::Float64Array,
        }
    }
}

/// A parameter value.
#[pyclass(name = "ParameterValue", module = "foxglove")]
#[derive(Clone)]
pub enum PyParameterValue {
    /// A decimal or integer value.
    Number(f64),
    /// A boolean value.
    Bool(bool),
    /// A byte array, which will be encoded as a base64-encoded string.
    Bytes(Vec<u8>),
    /// An array of parameter values.
    Array(Vec<PyParameterValue>),
    /// An associative map of parameter values.
    Dict(HashMap<String, PyParameterValue>),
}

impl From<PyParameterValue> for foxglove::websocket::ParameterValue {
    fn from(value: PyParameterValue) -> Self {
        match value {
            PyParameterValue::Number(n) => foxglove::websocket::ParameterValue::Number(n),
            PyParameterValue::Bool(b) => foxglove::websocket::ParameterValue::Bool(b),
            PyParameterValue::Bytes(items) => foxglove::websocket::ParameterValue::String(items),
            PyParameterValue::Array(py_parameter_values) => {
                foxglove::websocket::ParameterValue::Array(
                    py_parameter_values.into_iter().map(Into::into).collect(),
                )
            }
            PyParameterValue::Dict(hash_map) => foxglove::websocket::ParameterValue::Dict(
                hash_map.into_iter().map(|(k, v)| (k, v.into())).collect(),
            ),
        }
    }
}

impl From<foxglove::websocket::ParameterValue> for PyParameterValue {
    fn from(value: foxglove::websocket::ParameterValue) -> Self {
        match value {
            foxglove::websocket::ParameterValue::Number(n) => PyParameterValue::Number(n),
            foxglove::websocket::ParameterValue::Bool(b) => PyParameterValue::Bool(b),
            foxglove::websocket::ParameterValue::String(items) => PyParameterValue::Bytes(items),
            foxglove::websocket::ParameterValue::Array(parameter_values) => {
                PyParameterValue::Array(parameter_values.into_iter().map(Into::into).collect())
            }
            foxglove::websocket::ParameterValue::Dict(hash_map) => {
                PyParameterValue::Dict(hash_map.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
        }
    }
}

/// A parameter which can be sent to a client.
#[pyclass(name = "Parameter", module = "foxglove")]
#[derive(Clone)]
pub struct PyParameter {
    /// The name of the parameter.
    #[pyo3(get)]
    pub name: String,
    /// The parameter type.
    #[pyo3(get)]
    pub r#type: Option<PyParameterType>,
    /// The parameter value.
    #[pyo3(get)]
    pub value: Option<PyParameterValue>,
}

#[pymethods]
impl PyParameter {
    #[new]
    #[pyo3(signature = (name, *, r#type=None, value=None))]
    pub fn new(
        name: String,
        r#type: Option<PyParameterType>,
        value: Option<PyParameterValue>,
    ) -> Self {
        Self {
            name,
            r#type,
            value,
        }
    }
}

impl From<PyParameter> for foxglove::websocket::Parameter {
    fn from(value: PyParameter) -> Self {
        Self {
            name: value.name,
            r#type: value.r#type.map(Into::into),
            value: value.value.map(Into::into),
        }
    }
}

impl From<foxglove::websocket::Parameter> for PyParameter {
    fn from(value: foxglove::websocket::Parameter) -> Self {
        Self {
            name: value.name,
            r#type: value.r#type.map(Into::into),
            value: value.value.map(Into::into),
        }
    }
}
