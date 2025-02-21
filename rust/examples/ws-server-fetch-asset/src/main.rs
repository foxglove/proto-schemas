use clap::Parser;

use foxglove::websocket::{AssetResponder, Capability, ServerListener};
use std::collections::HashMap;
use std::sync::Arc;

struct AssetServer {
    assets: HashMap<String, Vec<u8>>,
}

impl AssetServer {
    fn new() -> Arc<Self> {
        let mut assets = HashMap::new();
        assets.insert("/test/one".to_string(), b"one".to_vec());
        assets.insert("/test/two".to_string(), b"two".to_vec());

        Arc::new(Self { assets })
    }
}

impl ServerListener for AssetServer {
    fn on_fetch_asset(&self, uri: String, responder: AssetResponder) {
        if let Some(asset) = self.assets.get(&uri) {
            // A real implementation might use std::fs::read to read a file into a Vec<u8>
            // The ws-protocol doesn't currently support streaming for a single asset.
            responder.send_data(asset);
        } else {
            responder.send_error(&format!("Asset {} not found", uri));
        }
    }
}

#[derive(Debug, Parser)]
struct Cli {
    /// Server TCP port.
    #[arg(short, long, default_value_t = 8765)]
    port: u16,
    /// Server IP address.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() {
    let env = env_logger::Env::default().default_filter_or("debug");
    env_logger::init_from_env(env);

    let args = Cli::parse();

    let asset_server = AssetServer::new();

    let server = foxglove::WebSocketServer::new()
        .name("ws-demo")
        .bind(&args.host, args.port)
        .listener(asset_server)
        .capabilities([Capability::Assets])
        .start()
        .await
        .expect("Server failed to start");

    tokio::signal::ctrl_c().await.ok();
    server.stop().await;
}
