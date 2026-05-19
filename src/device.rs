use crate::memory::Memory;

pub trait Device {
    fn step(&mut self, mem: &mut Memory);
}
