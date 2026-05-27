use crate::bus::Bus;

/// The trait that all devices connected to the [`Bus`] implement.
pub trait Device {
    /// Notifies the device that a new clock cycle has begun.
    fn tick(&mut self, bus: &mut Bus);
}
