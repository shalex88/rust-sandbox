mod api;
mod core;
mod infra;

use api::factory::{self as api_factory, ApiType};
use core::Core;
use infra::device::factory::{self as device_factory, DeviceType};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let device = device_factory::create(DeviceType::Fake);
    let core = Core::new(device);
    let api = api_factory::create(ApiType::Rest, core);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on http://{}", listener.local_addr()?);
    api.serve(listener).await
}
