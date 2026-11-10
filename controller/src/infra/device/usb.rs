use super::Device;

/// Simulated USB device; does not connect to hardware.
pub struct UsbDevice {
    name: String,
}

impl Default for UsbDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl UsbDevice {
    pub fn new() -> Self {
        Self {
            name: String::from("USB Device"),
        }
    }
}

impl Device for UsbDevice {
    fn name(&self) -> &str {
        &self.name
    }
}
