use std::fmt::Display;

use bytes::Bytes;

use crate::websocket::service::{Request, Responder};
use crate::websocket::Client;

/// A websocket service call handler.
pub trait Handler: Send + Sync {
    /// Handles a service call request from a client.
    ///
    /// The caller can choose whether to handle the call synchronously or asynchronously.
    ///
    /// This method is invoked from the client's main poll loop and must not block. If blocking or
    /// long-running behavior is required, the implementation should use [`tokio::task::spawn`] (or
    /// [`tokio::task::spawn_blocking`]) to handle the request asynchronously.
    ///
    /// The implementation is responsible for completing the request with [`Responder::respond`],
    /// otherwise no response will be sent to the client.
    fn call(&self, client: Client, request: Request, responder: Responder);
}

/// A wrapper around a function that serves as a service call handler.
pub(crate) struct HandlerFn<F>(pub F)
where
    F: Fn(Client, Request, Responder) + Send + Sync;

impl<F> Handler for HandlerFn<F>
where
    F: Fn(Client, Request, Responder) + Send + Sync,
{
    fn call(&self, client: Client, request: Request, responder: Responder) {
        self.0(client, request, responder);
    }
}

/// A synchronous service call handler.
///
/// This is a convenience wrapper around [`Handler`] that takes care of moving the result into the
/// [`Responder`], so that the implementation can take advantage of standard control flow idioms
/// for returning errors.
pub trait SyncHandler: Send + Sync {
    /// The error type returned for service calls.
    type Error: Display;

    /// Synchronously handles a service call request from a client and returns a result.
    ///
    /// This method is invoked from the client's main poll loop and must not block. If blocking or
    /// long-running behavior is required, use [`Handler`] instead.
    fn call(&self, client: Client, request: Request) -> Result<Bytes, Self::Error>;
}

impl<T: SyncHandler> Handler for T {
    fn call(&self, client: Client, request: Request, responder: Responder) {
        let result = SyncHandler::call(self, client, request);
        responder.respond(result.map_err(|e| e.to_string()));
    }
}

/// A wrapper around a function that serves as a synchronous service call handler.
pub(crate) struct SyncHandlerFn<F, E>(pub F)
where
    F: Fn(Client, Request) -> Result<Bytes, E> + Send + Sync,
    E: Display + 'static;

impl<F, E> SyncHandler for SyncHandlerFn<F, E>
where
    F: Fn(Client, Request) -> Result<Bytes, E> + Send + Sync,
    E: Display + 'static,
{
    type Error = E;

    fn call(&self, client: Client, request: Request) -> Result<Bytes, Self::Error> {
        self.0(client, request)
    }
}
