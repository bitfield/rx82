use crate::{bus::Bus, cpu::Phase};

/// A snapshot of the system state for debugging.
#[non_exhaustive]
pub struct Snapshot {
    pub bus: Bus,
    pub phase: Phase,
    pub tick: u16,
}
