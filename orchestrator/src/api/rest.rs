use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, Uri},
    routing::get,
};
use serde::Serialize;

use super::Api;
use crate::core::{Core, Error};

pub struct RestApi {
    api: Arc<Api>,
}

impl RestApi {
    pub fn new(api: Arc<Api>) -> Self {
        Self { api }
    }

    pub async fn serve(self, listener: tokio::net::TcpListener) -> std::io::Result<()> {
        axum::serve(listener, routes(self.api)).await
    }
}

pub fn router(core: Core) -> Router {
    routes(Arc::new(Api::new(core)))
}

fn routes(api: Arc<Api>) -> Router {
    Router::new()
        .route("/devices/{id}/name", get(device_name))
        .with_state(api)
}

#[derive(Serialize)]
struct DeviceNameResponse {
    name: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
}

async fn device_name(
    State(api): State<Arc<Api>>,
    Path(id): Path<String>,
    _uri: Uri,
) -> Result<Json<DeviceNameResponse>, (StatusCode, Json<ErrorResponse>)> {
    #[cfg(debug_assertions)]
    println!("Request: {}", _uri.path());
    api.device_name(&id)
        .await
        .map(|name| {
            #[cfg(debug_assertions)]
            println!("Response: {}", name);
            Json(DeviceNameResponse { name })
        })
        .map_err(|error| {
            let (status, message) = match error {
                Error::UnknownDevice => (StatusCode::NOT_FOUND, "unknown device"),
                Error::DeviceServiceFailed => {
                    (StatusCode::BAD_GATEWAY, "device service request failed")
                }
                Error::Timeout => (StatusCode::GATEWAY_TIMEOUT, "device service timed out"),
            };
            #[cfg(debug_assertions)]
            println!("Response: {}", message);
            (status, Json(ErrorResponse { error: message }))
        })
}
