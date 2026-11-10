use std::{future::Future, pin::Pin, time::Duration};

use axum::{
    body::to_bytes,
    http::{Request, Uri, header::HOST},
};
use http_body_util::Empty;
use hyper::{body::Bytes, client::conn::http1};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use tokio::{net::TcpStream, time::timeout};

use crate::core::{DeviceService, Error};

const MAX_RESPONSE_BYTES: usize = 64 * 1024;

pub(super) struct HttpDeviceService {
    pub endpoint: Uri,
    pub timeout: Duration,
}

#[derive(Deserialize)]
struct DeviceNameResponse {
    name: String,
}

impl HttpDeviceService {
    async fn fetch_name(&self) -> Result<String, Error> {
        let authority = self
            .endpoint
            .authority()
            .ok_or(Error::DeviceServiceFailed)?;
        // Uri::host includes brackets for IPv6; Tokio expects the bare address.
        let host = authority
            .host()
            .trim_start_matches('[')
            .trim_end_matches(']');
        let stream = TcpStream::connect((host, authority.port_u16().unwrap_or(80)))
            .await
            .map_err(|_| Error::DeviceServiceFailed)?;
        let (mut sender, connection) = http1::handshake(TokioIo::new(stream))
            .await
            .map_err(|_| Error::DeviceServiceFailed)?;
        let request = Request::builder()
            .uri(
                self.endpoint
                    .path_and_query()
                    .ok_or(Error::DeviceServiceFailed)?
                    .as_str(),
            )
            .header(HOST, authority.as_str())
            .body(Empty::<Bytes>::new())
            .map_err(|_| Error::DeviceServiceFailed)?;
        let exchange = async {
            let response = sender
                .send_request(request)
                .await
                .map_err(|_| Error::DeviceServiceFailed)?;
            if !response.status().is_success() {
                return Err(Error::DeviceServiceFailed);
            }
            let bytes = to_bytes(
                axum::body::Body::new(response.into_body()),
                MAX_RESPONSE_BYTES,
            )
            .await
            .map_err(|_| Error::DeviceServiceFailed)?;
            let response: DeviceNameResponse =
                serde_json::from_slice(&bytes).map_err(|_| Error::DeviceServiceFailed)?;
            Ok(response.name)
        };
        tokio::pin!(exchange);
        // Drive HTTP and consume the body together without leaving background tasks
        // or connections behind if the outer timeout cancels this request.
        tokio::select! {
            result = &mut exchange => result,
            result = connection => {
                result.map_err(|_| Error::DeviceServiceFailed)?;
                exchange.await
            }
        }
    }
}

impl DeviceService for HttpDeviceService {
    fn device_name(&self) -> Pin<Box<dyn Future<Output = Result<String, Error>> + Send + '_>> {
        Box::pin(async move {
            timeout(self.timeout, self.fetch_name())
                .await
                .map_err(|_| Error::Timeout)?
        })
    }
}
