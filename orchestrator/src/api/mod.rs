pub mod factory;
pub mod rest;

use crate::core::{Core, Error};

/// Application API shared by transport implementations.
pub struct Api {
    core: Core,
}

impl Api {
    pub fn new(core: Core) -> Self {
        Self { core }
    }

    pub async fn device_name(&self, id: &str) -> Result<String, Error> {
        self.core.device_name(id).await
    }
}
