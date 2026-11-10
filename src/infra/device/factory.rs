use super::{Device, FakeDevice, UsbDevice};

/// Device implementation to create.
pub enum DeviceType {
    Fake,
    Usb,
}

/// A configured device, usable through the common device interface.
pub enum DeviceInstance {
    Fake(FakeDevice),
    Usb(UsbDevice),
}

impl Device for DeviceInstance {
    fn name(&self) -> &str {
        match self {
            Self::Fake(device) => device.name(),
            Self::Usb(device) => device.name(),
        }
    }
}

pub fn create(device_type: DeviceType) -> DeviceInstance {
    match device_type {
        DeviceType::Fake => DeviceInstance::Fake(FakeDevice::new()),
        DeviceType::Usb => DeviceInstance::Usb(UsbDevice::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::{DeviceType, create};
    use crate::core::Core;

    #[test]
    fn creates_devices_usable_by_core() {
        for (device_type, expected_name) in [
            (DeviceType::Fake, "Fake Device"),
            (DeviceType::Usb, "USB Device"),
        ] {
            let core = Core::new(create(device_type));
            assert_eq!(core.device_name(), expected_name);
        }
    }
}
