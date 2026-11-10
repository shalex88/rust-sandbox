use std::{env, fs, io, time::Duration};

use orchestrator::{
    api::factory::{self, ApiType},
    infra::registry::RegistryConfig,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let registry_path =
        env::var_os("DEVICE_REGISTRY").unwrap_or_else(|| "orchestrator/devices.json".into());
    let config = RegistryConfig::parse(&fs::read_to_string(registry_path)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let address = env::var("ORCHESTRATOR_ADDR").unwrap_or_else(|_| "127.0.0.1:3001".into());
    let api = factory::create(ApiType::Rest, config.build(Duration::from_secs(5)));
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!(
        "Orchestrator listening on http://{}",
        listener.local_addr()?
    );
    api.serve(listener).await
}
