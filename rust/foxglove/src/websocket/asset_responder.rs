use super::Client;

/// A helper for sending asset responses to a client.
pub struct AssetResponder {
    /// The client requesting the asset.
    pub client: Client,
    request_id: u32,
}

impl AssetResponder {
    /// Create a new asset responder for a fetch asset request.
    pub(crate) fn new(client: Client, request_id: u32) -> Self {
        Self { client, request_id }
    }

    /// Send an error response to the client.
    pub fn send_error(&self, error: &str) {
        self.client.send_asset_error(error, self.request_id);
    }

    /// Send a successful response to the client with the asset.
    pub fn send_data(&self, asset: &[u8]) {
        self.client.send_asset_response(asset, self.request_id);
    }
}
