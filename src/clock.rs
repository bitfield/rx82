use core::time::Duration;
use std::{thread::sleep, time::Instant};

use crate::{bus::Bus, device::Device};

#[non_exhaustive]
pub struct Clock {
    pub next_tick: Instant,
    pub tick_duration: Duration,
}

impl Default for Clock {
    #[inline]
    fn default() -> Self {
        Self {
            next_tick: Instant::now(),
            tick_duration: Duration::from_nanos(250), // 4MHz
        }
    }
}

impl Device for Clock {
    #[inline]
    fn tick(&mut self, _bus: &mut Bus) {
        let now = Instant::now();
        self.next_tick = self
            .next_tick
            .checked_add(self.tick_duration)
            .unwrap_or(now); // too far in the future
        let wait = self.next_tick.saturating_duration_since(now);
        if !wait.is_zero() {
            sleep(wait);
        }
    }
}
