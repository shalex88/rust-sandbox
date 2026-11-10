use crate::infra::device::Device;

/// Application logic, independent of HTTP and the hardware implementation.
pub struct Core {
    device: Box<dyn Device>,
}

impl Core {
    pub fn new(device: impl Device + 'static) -> Self {
        Self {
            device: Box::new(device),
        }
    }

    pub fn device_name(&self) -> &str {
        self.device.name()
    }
}

#[cfg(test)]
mod tests {
    use super::Core;
    use crate::infra::device::Device;

    struct TestDevice;

    impl Device for TestDevice {
        fn name(&self) -> &str {
            "Test sensor"
        }
    }

    #[test]
    fn returns_the_device_name() {
        let core = Core::new(TestDevice);

        assert_eq!(core.device_name(), "Test sensor");
    }
}
