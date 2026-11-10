use std::sync::Arc;

use tokio::net::TcpListener;

use super::{Api, rest::RestApi};
use crate::core::Core;

/// Transport to expose the shared application API through.
pub enum ApiType {
    Rest,
}

/// A configured API server. Each transport owns its serving implementation.
pub enum ApiServer {
    Rest(RestApi),
}

impl ApiServer {
    pub async fn serve(self, listener: TcpListener) -> std::io::Result<()> {
        match self {
            Self::Rest(server) => server.serve(listener).await,
        }
    }
}

pub fn create(api_type: ApiType, core: Core) -> ApiServer {
    let api = Arc::new(Api::new(core));
    match api_type {
        ApiType::Rest => ApiServer::Rest(RestApi::new(api)),
    }
}
