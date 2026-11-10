use std::sync::Arc;

use axum::{Json, Router, extract::State, http::Uri, routing::get};
use serde::Serialize;

use crate::api::Api;

pub struct RestApi {
    api: Arc<Api>,
}

impl RestApi {
    pub fn new(api: Arc<Api>) -> Self {
        Self { api }
    }

    pub async fn serve(self, listener: tokio::net::TcpListener) -> std::io::Result<()> {
        axum::serve(listener, router(self.api)).await
    }
}

fn router(api: Arc<Api>) -> Router {
    Router::new()
        .route("/device/name", get(device_name))
        .with_state(api)
}

#[derive(Serialize)]
struct DeviceNameResponse {
    name: String,
}

async fn device_name(State(api): State<Arc<Api>>, _uri: Uri) -> Json<DeviceNameResponse> {
    #[cfg(debug_assertions)]
    println!("Request: {}", _uri.path());
    let response = DeviceNameResponse {
        name: api.device_name().to_owned(),
    };
    #[cfg(debug_assertions)]
    println!("Response: {}", response.name);
    Json(response)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use std::sync::Arc;

    use crate::{
        api::Api,
        core::Core,
        infra::device::{Device, FakeDevice},
    };

    struct NamedTestDevice(&'static str);

    impl Device for NamedTestDevice {
        fn name(&self) -> &str {
            self.0
        }
    }

    #[tokio::test]
    async fn returns_device_name_as_json() {
        let cases = [
            (Core::new(FakeDevice::new()), "Fake Device"),
            (Core::new(NamedTestDevice("Lab device 42")), "Lab device 42"),
            (
                Core::new(NamedTestDevice("Device \"α\"\nUSB")),
                "Device \"α\"\nUSB",
            ),
        ];
        for (core, name) in cases {
            let response = super::router(Arc::new(Api::new(core)))
                .oneshot(
                    Request::builder()
                        .uri("/device/name")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()["content-type"], "application/json");
            let body = to_bytes(response.into_body(), 1024).await.unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(json, serde_json::json!({"name": name}));
        }
    }
}
