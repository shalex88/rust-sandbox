pub mod factory;
mod fake;
mod usb;

pub use fake::FakeDevice;
pub use usb::UsbDevice;

/// Common interface for infrastructure device implementations.
pub trait Device: Send + Sync {
    fn name(&self) -> &str;
}
