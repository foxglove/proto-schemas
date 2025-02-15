//! Websocket service request.

use std::sync::Arc;

use bytes::Bytes;

use super::{CallId, Service, ServiceId};

/// A service call request.
#[derive(Debug, Clone)]
pub struct Request {
    service: Arc<Service>,
    call_id: CallId,
    encoding: String,
    payload: Bytes,
}

impl Request {
    /// Constructs a new request.
    pub(crate) fn new(
        service: Arc<Service>,
        call_id: CallId,
        encoding: String,
        payload: Bytes,
    ) -> Self {
        Self {
            service,
            call_id,
            encoding,
            payload,
        }
    }

    /// The service ID.
    pub fn service_id(&self) -> ServiceId {
        self.service.id()
    }

    /// The service name.
    pub fn service_name(&self) -> &str {
        self.service.name()
    }

    /// The call ID that uniquely identifies this request for this client.
    pub fn call_id(&self) -> CallId {
        self.call_id
    }

    /// The request encoding.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// A reference to the request payload.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Consumes the request to return the inner payload.
    pub fn into_payload(self) -> Bytes {
        self.payload
    }
}
