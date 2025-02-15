use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use bytes::Bytes;
use foxglove::websocket::service::{Request, Service, ServiceSchema, SyncHandler};
use foxglove::websocket::{Capability, Client};
use foxglove::Schema;
use tracing::info;

use crate::types::{IntBinRequest, IntBinResponse, SetBoolRequest, SetBoolResponse};
use crate::Config;

pub async fn main(config: Config) -> Result<()> {
    let server = foxglove::WebSocketServer::new()
        .name("echo")
        .bind(&config.host, config.port)
        .capabilities([Capability::Services])
        .supported_encodings(["raw"])
        .start()
        .await
        .context("Failed to start server")?;

    // Simple services can be implemented with a closure.
    server
        .add_services([
            Service::builder("/empty", empty_schema())
                .sync_handler_fn(|_, _| anyhow::Ok(Bytes::new())),
            Service::builder("/echo", echo_schema())
                .sync_handler_fn(|_, req| anyhow::Ok(req.into_payload())),
        ])
        .context("Failed to register services")?;

    // Services that need to sleep (or do heavy computation) should use `tokio::spawn()`
    // (or `tokio::task::spawn_blocking()`) to avoid blocking the client's main event loop.
    // Unlike the `SyncHandler` implementations, the generic `Handler` is responsible for
    // invoking `resp.respond()` to complete the request.
    server
        .add_services([
            Service::builder("/sleep", empty_schema()).handler_fn(|_, _, resp| {
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    resp.respond(Ok(Bytes::new()))
                });
            }),
        ])
        .context("Failed to register services")?;

    // A single handler function can be shared by multiple services.
    server
        .add_services(
            ["/IntBin/add", "/IntBin/sub", "/IntBin/mul", "/IntBin/mod"]
                .into_iter()
                .map(|name| {
                    Service::builder(name, int_bin_schema()).sync_handler_fn(int_bin_handler)
                }),
        )
        .context("Failed to register services")?;

    // A stateful handler might be written as a type that implements `Handler` (or `SyncHandler`).
    let flag_a = Flag::default();
    let flag_b = Flag::default();
    server
        .add_services([
            Service::builder("/flag_a", set_bool_schema()).handler(flag_a.clone()),
            Service::builder("/flag_b", set_bool_schema()).handler(flag_b.clone()),
        ])
        .context("Failed to register services")?;

    tokio::signal::ctrl_c().await.ok();
    server.stop().await;
    Ok(())
}

fn empty_schema() -> ServiceSchema {
    // A simple schema with a "well-known" request & response.
    ServiceSchema::new("/std_srvs/Empty")
}

fn echo_schema() -> ServiceSchema {
    // A simple schema with a well-specified request & response.
    ServiceSchema::new("/custom_srvs/RawEcho")
        .with_request("raw", Schema::new("raw", "none", b""))
        .with_response("raw", Schema::new("raw", "none", b""))
}

fn int_bin_schema() -> ServiceSchema {
    // Schemas can be derived from types that implement `JsonSchema` using the
    // `Schema::json_schema()` method.
    ServiceSchema::new("/custom_srvs/IntBinOps")
        .with_request("json", Schema::json_schema::<IntBinRequest>())
        .with_response("json", Schema::json_schema::<IntBinResponse>())
}

fn set_bool_schema() -> ServiceSchema {
    ServiceSchema::new("/std_srvs/SetBool")
        .with_request("json", Schema::json_schema::<SetBoolRequest>())
        .with_response("json", Schema::json_schema::<SetBoolResponse>())
}

/// A stateless handler function.
fn int_bin_handler(client: Client, req: Request) -> Result<Bytes> {
    let service_name = req.service_name();
    let req: IntBinRequest = serde_json::from_slice(req.payload())?;
    info!("Client {:?}: {service_name}: {req:?}", client.id());

    // Shared handlers can use `Request::service_name` to disambiguate the service endpoint.
    // Service names are guaranteed to be unique.
    let result = match service_name {
        "/IntBin/add" => req.a + req.b,
        "/IntBin/sub" => req.a - req.b,
        "/IntBin/mul" => req.a * req.b,
        "/IntBin/mod" => req.a % req.b,
        m => return Err(anyhow::anyhow!("unexpected service: {m}")),
    };

    let payload = serde_json::to_vec(&IntBinResponse { result })?;
    Ok(payload.into())
}

/// A stateful handler implements the `SyncHandler` trait.
#[derive(Debug, Default, Clone)]
struct Flag(Arc<AtomicBool>);

impl SyncHandler for Flag {
    type Error = anyhow::Error;

    fn call(&self, client: Client, req: Request) -> Result<Bytes, Self::Error> {
        // Decode the payload.
        let req: SetBoolRequest = serde_json::from_slice(req.payload())?;
        info!("Client {:?}: {req:?}", client.id());

        // Update the flag.
        let prev = self.0.swap(req.data, std::sync::atomic::Ordering::Relaxed);

        // Encode the response.
        let message = if prev == req.data {
            "unchanged".to_string()
        } else {
            format!("updated {prev} -> {}", req.data)
        };
        let payload = serde_json::to_vec(&SetBoolResponse {
            success: true,
            message,
        })?;

        Ok(payload.into())
    }
}
