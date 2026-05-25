use crate::{bus::Bus, device::Device};

#[non_exhaustive]
#[derive(Default)]
pub struct Clock;

impl Device for Clock {
    #[inline]
    fn tick(&mut self, _bus: &mut Bus) {
        println!("Tick!");
    }
}
