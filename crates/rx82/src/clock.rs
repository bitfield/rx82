use core::time::Duration;
use std::{thread::sleep, time::Instant};

use crate::{bus::Bus, system::Device};

/// The system clock.
///
/// Does nothing except try to slow down the system to the nominal cycle rate by
/// delaying its return from the [`Clock::tick`] method.
pub struct Clock {
    /// Time the next tick is due.
    pub next_tick: Instant,
    /// Target duration for each tick.
    pub tick_duration: Duration,
}

impl Default for Clock {
    /// Creates a default [`Clock`] with a nominal frequency of 4MHz.
    fn default() -> Self {
        Self {
            next_tick: Instant::now(),
            tick_duration: Duration::from_nanos(250), // 4MHz
        }
    }
}

impl Device for Clock {
    /// Waits until the next tick is due.
    ///
    /// If the projected next tick time overflows `usize`, or we are already past the
    /// next tick time, returns immediately.
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
