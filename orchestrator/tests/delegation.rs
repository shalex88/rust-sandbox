use std::time::Duration;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    routing::get,
};
use orchestrator::{api::rest::router, infra::registry::RegistryConfig};
use serde_json::{Value, json};
use tower::ServiceExt;

struct Server {
    url: String,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve(app: Router) -> Server {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    Server { url, task }
}

fn app(entries: Value, timeout: Duration) -> Router {
    let config = RegistryConfig::parse(&entries.to_string()).unwrap();
    router(config.build(timeout))
}

async fn request(app: Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    assert_eq!(response.headers()["content-type"], "application/json");
    let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn selects_device_services_and_preserves_json_and_url_prefixes() {
    let first = serve(Router::new().route(
        "/device/name",
        get(|| async { axum::Json(json!({"name": "Device \"α\"\nUSB"})) }),
    ))
    .await;
    let second = serve(Router::new().route(
        "/lab@one/device/name",
        get(|| async { axum::Json(json!({"name": "Second device"})) }),
    ))
    .await;
    let app = app(
        json!([
            {"id": "first", "url": first.url},
            {"id": "second", "url": format!("{}/lab@one/", second.url)}
        ]),
        Duration::from_secs(1),
    );
    for (id, name) in [("first", "Device \"α\"\nUSB"), ("second", "Second device")] {
        let (status, body) = request(app.clone(), &format!("/devices/{id}/name")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({"name": name}));
    }
    let (status, body) = request(app, "/devices/missing/name").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body, json!({"error": "unknown device"}));
}

#[tokio::test]
async fn maps_upstream_errors_to_bad_gateway_without_leaking_details() {
    for (status, body) in [
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "private upstream details".to_owned(),
        ),
        (StatusCode::NOT_FOUND, "missing".to_owned()),
        (StatusCode::FOUND, "redirect".to_owned()),
        (StatusCode::OK, "not JSON".to_owned()),
        (StatusCode::OK, "{}".to_owned()),
        (StatusCode::OK, r#"{"name":42}"#.to_owned()),
        (
            StatusCode::OK,
            format!(r#"{{"name":"{}"}}"#, "x".repeat(65536)),
        ),
    ] {
        let upstream = serve(Router::new().route(
            "/device/name",
            get(move || {
                let body = body.clone();
                async move { (status, body) }
            }),
        ))
        .await;
        let (actual, body) = request(
            app(
                json!([{"id": "lab", "url": upstream.url}]),
                Duration::from_secs(1),
            ),
            "/devices/lab/name",
        )
        .await;
        assert_eq!(actual, StatusCode::BAD_GATEWAY);
        assert_eq!(body, json!({"error": "device service request failed"}));
    }
}

#[tokio::test]
async fn connection_failures_return_bad_gateway() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let (status, _) = request(
        app(
            json!([{"id": "offline", "url": url}]),
            Duration::from_secs(1),
        ),
        "/devices/offline/name",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn slow_device_returns_gateway_timeout() {
    let upstream = serve(Router::new().route(
        "/device/name",
        get(|| async {
            std::future::pending::<()>().await;
            "unreachable"
        }),
    ))
    .await;
    let (status, body) = request(
        app(
            json!([{"id": "slow", "url": upstream.url}]),
            Duration::from_millis(30),
        ),
        "/devices/slow/name",
    )
    .await;
    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
    assert_eq!(body, json!({"error": "device service timed out"}));
}

#[tokio::test]
async fn timeout_also_covers_reading_the_response_body() {
    // Headers arrive immediately, but this device never finishes its body.
    struct PendingBody;
    impl hyper::body::Body for PendingBody {
        type Data = hyper::body::Bytes;
        type Error = std::convert::Infallible;

        fn poll_frame(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Result<hyper::body::Frame<Self::Data>, Self::Error>>> {
            std::task::Poll::Pending
        }
    }

    let upstream =
        serve(Router::new().route("/device/name", get(|| async { Body::new(PendingBody) }))).await;
    let (status, _) = request(
        app(
            json!([{"id": "slow", "url": upstream.url}]),
            Duration::from_millis(30),
        ),
        "/devices/slow/name",
    )
    .await;
    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
}

#[test]
fn rejects_invalid_registry_entries() {
    for entries in [
        json!([{"id": "lab", "url": "http://localhost:3000"}, {"id": "lab", "url": "http://localhost:4000"}]),
        json!([{"id": "", "url": "http://localhost:3000"}]),
        json!([{"id": "with/slash", "url": "http://localhost:3000"}]),
        json!([{"id": "lab", "url": "not a URL"}]),
        json!([{"id": "lab", "url": "https://localhost:3000"}]),
        json!([{"id": "lab", "url": "http://localhost:3000?query=1"}]),
        json!([{"id": "lab", "url": "http://localhost:3000/#fragment"}]),
        json!([{"id": "lab", "url": "http://user:password@localhost:3000"}]),
        json!([{"id": "lab", "url": "http://localhost:99999"}]),
        json!([{"id": "lab", "url": "http://[not-an-ipv6]"}]),
        json!([{"id": "lab", "url": "http://[::zzz]"}]),
        json!([{"id": "lab", "url": "http://localhost:3000", "typo": true}]),
    ] {
        assert!(
            RegistryConfig::parse(&entries.to_string()).is_err(),
            "accepted {entries}"
        );
    }
    assert!(RegistryConfig::parse("{").is_err());
    assert!(RegistryConfig::parse("[]").is_ok());
    assert!(RegistryConfig::parse(r#"[{"id":"lab-1_usb","url":"http://[::1]:3000/"}]"#).is_ok());
}
