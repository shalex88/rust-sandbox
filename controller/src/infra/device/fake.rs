use super::Device;

/// In-memory stand-in for hardware.
pub struct FakeDevice {
    name: String,
}

impl Default for FakeDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeDevice {
    pub fn new() -> Self {
        Self {
            name: String::from("Fake Device"),
        }
    }
}

impl Device for FakeDevice {
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::{Device, FakeDevice};

    #[test]
    fn returns_its_name() {
        let device = FakeDevice::new();

        assert_eq!(device.name(), "Fake Device");
    }
}
