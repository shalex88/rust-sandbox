use std::{collections::BTreeMap, future::Future, pin::Pin};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnknownDevice,
    DeviceServiceFailed,
    Timeout,
}

/// Port implemented by infrastructure; core has no HTTP or URL dependency.
pub trait DeviceService: Send + Sync {
    fn device_name(&self) -> Pin<Box<dyn Future<Output = Result<String, Error>> + Send + '_>>;
}

pub struct Core {
    services: BTreeMap<String, Box<dyn DeviceService>>,
}

impl Core {
    pub fn new(services: BTreeMap<String, Box<dyn DeviceService>>) -> Self {
        Self { services }
    }

    pub async fn device_name(&self, id: &str) -> Result<String, Error> {
        self.services
            .get(id)
            .ok_or(Error::UnknownDevice)?
            .device_name()
            .await
    }
}
