use crate::bus::Bus;

pub trait Device {
    fn tick(&mut self, bus: &mut Bus);
}
