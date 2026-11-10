use std::env;

use controller::{
    api::factory::{self as api_factory, ApiType},
    core::Core,
    infra::device::factory::{self as device_factory, DeviceType},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let device = device_factory::create(DeviceType::Fake);
    let core = Core::new(device);
    let api = api_factory::create(ApiType::Rest, core);

    let address = env::var("CONTROLLER_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("Listening on http://{}", listener.local_addr()?);
    api.serve(listener).await
}
