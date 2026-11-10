pub mod factory;
mod rest;

use crate::core::Core;

/// Shared application API, independent of REST and other transports.
pub struct Api {
    core: Core,
}

impl Api {
    pub fn new(core: Core) -> Self {
        Self { core }
    }

    pub fn device_name(&self) -> &str {
        self.core.device_name()
    }
}
