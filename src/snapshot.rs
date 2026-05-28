use crate::{bus::Bus, phase::Phase};

#[non_exhaustive]
pub struct Snapshot {
    pub bus: Bus,
    pub phase: Phase,
    pub tick: u16,
}
