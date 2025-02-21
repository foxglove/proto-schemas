//! Websocket services.

use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::websocket::Client;

mod handler;
mod request;
mod response;
mod schema;
mod semaphore;
pub use handler::{Handler, SyncHandler};
use handler::{HandlerFn, SyncHandlerFn};
pub use request::Request;
pub use response::Responder;
pub(crate) use schema::MessageSchema;
pub use schema::ServiceSchema;
pub(crate) use semaphore::Semaphore;

/// A service ID, which uniquely identifies a service hosted by the server.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct ServiceId(u32);

impl ServiceId {
    /// Creates a new service ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl From<ServiceId> for u32 {
    fn from(id: ServiceId) -> u32 {
        id.0
    }
}

impl Display for ServiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A service call ID, which uniquely identifies an outstanding call for a particular client.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
pub struct CallId(u32);

impl CallId {
    /// Creates a new service ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl From<CallId> for u32 {
    fn from(id: CallId) -> u32 {
        id.0
    }
}

impl Display for CallId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A builder for a websocket service.
#[must_use]
#[derive(Debug)]
pub struct ServiceBuilder {
    id: ServiceId,
    name: String,
    schema: ServiceSchema,
}
impl ServiceBuilder {
    /// Creates a new builder for a websocket service.
    fn new(name: impl Into<String>, schema: ServiceSchema) -> Self {
        static ID: AtomicU32 = AtomicU32::new(1);
        let id = ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id: ServiceId::new(id),
            name: name.into(),
            schema,
        }
    }

    /// Allow overriding the ID for deterministic tests.
    #[cfg(test)]
    pub(crate) fn with_id(mut self, id: ServiceId) -> Self {
        self.id = id;
        self
    }

    /// Configures a handler and returns the constructed [`Service`].
    pub fn handler<H: Handler + 'static>(self, handler: H) -> Service {
        Service {
            id: self.id,
            name: self.name,
            schema: self.schema,
            handler: Arc::new(handler),
        }
    }

    /// Configures a handler function and returns the constructed [`Service`].
    ///
    /// Refer to [`Handler::call`] for a description of the `call` function.
    pub fn handler_fn<F>(self, call: F) -> Service
    where
        F: Fn(Client, Request, Responder) + Send + Sync + 'static,
    {
        self.handler(HandlerFn(call))
    }

    /// Configures a synchronous handler function and returns the constructed [`Service`].
    ///
    /// Refer to [`SyncHandler::call`] for a description of the `call` function.
    pub fn sync_handler_fn<F, E>(self, call: F) -> Service
    where
        F: Fn(Client, Request) -> Result<Bytes, E> + Send + Sync + 'static,
        E: Display + 'static,
    {
        self.handler(SyncHandlerFn(call))
    }
}

/// A websocket service.
#[must_use]
pub struct Service {
    id: ServiceId,
    name: String,
    schema: ServiceSchema,
    handler: Arc<dyn Handler>,
}

impl std::fmt::Debug for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Service")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("schema", &self.schema)
            .finish_non_exhaustive()
    }
}

impl Service {
    /// Creates a new builder for a websocket service.
    pub fn builder(name: impl Into<String>, schema: ServiceSchema) -> ServiceBuilder {
        ServiceBuilder::new(name, schema)
    }

    /// Returns the service's ID.
    pub fn id(&self) -> ServiceId {
        self.id
    }

    /// Returns the service's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the service schema.
    pub fn schema(&self) -> &ServiceSchema {
        &self.schema
    }

    /// The declared request encoding.
    pub(crate) fn request_encoding(&self) -> Option<&str> {
        self.schema().request().map(|rs| rs.encoding.as_str())
    }

    /// The declared repsonse encoding.
    pub(crate) fn response_encoding(&self) -> Option<&str> {
        self.schema().response().map(|rs| rs.encoding.as_str())
    }

    /// Invokes the service call implementation.
    pub(crate) fn call(&self, client: Client, request: Request, responder: Responder) {
        self.handler.call(client, request, responder);
    }
}
