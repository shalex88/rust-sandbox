use std::{collections::BTreeMap, net::Ipv6Addr, time::Duration};

use axum::http::Uri;
use serde::Deserialize;

use super::device_service::HttpDeviceService;
use crate::core::{Core, DeviceService};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: String,
    url: String,
}

/// Validated configuration. All entries are checked before any service starts.
pub struct RegistryConfig {
    endpoints: BTreeMap<String, Uri>,
}

impl RegistryConfig {
    pub fn parse(json: &str) -> Result<Self, String> {
        let entries: Vec<Entry> = serde_json::from_str(json).map_err(|error| error.to_string())?;
        let mut endpoints = BTreeMap::new();
        for Entry { id, url } in entries {
            if id.is_empty()
                || !id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
            {
                return Err(
                    "device IDs must contain only ASCII letters, digits, '-' or '_'".into(),
                );
            }
            let invalid_url = || {
                format!("device '{id}' requires an HTTP URL without credentials, query or fragment")
            };
            if url.contains(['?', '#']) {
                return Err(invalid_url());
            }
            let base: Uri = url.parse().map_err(|_| invalid_url())?;
            let authority = base.authority().ok_or_else(invalid_url)?;
            if base.scheme_str() != Some("http")
                || authority.as_str().contains('@')
                || authority.host().is_empty()
                || (authority.as_str().len() > authority.host().len()
                    && authority.port_u16().is_none())
            {
                return Err(invalid_url());
            }
            if let Some(host) = authority.host().strip_prefix('[') {
                host.strip_suffix(']')
                    .and_then(|host| host.parse::<Ipv6Addr>().ok())
                    .ok_or_else(invalid_url)?;
            }
            let endpoint: Uri = format!("{}/device/name", url.trim_end_matches('/'))
                .parse()
                .map_err(|_| invalid_url())?;
            if endpoints.insert(id.clone(), endpoint).is_some() {
                return Err(format!("duplicate device ID '{id}'"));
            }
        }
        Ok(Self { endpoints })
    }

    pub fn build(self, timeout: Duration) -> Core {
        let services = self
            .endpoints
            .into_iter()
            .map(|(id, endpoint)| {
                (
                    id,
                    Box::new(HttpDeviceService { endpoint, timeout }) as Box<dyn DeviceService>,
                )
            })
            .collect();
        Core::new(services)
    }
}
