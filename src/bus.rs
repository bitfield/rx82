use anyhow::{Result, ensure};

/// The system bus.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
#[expect(clippy::partial_pub_fields, reason = "pending_write is internal")]
pub struct Bus {
    /// The 16-bit address bus.
    pub addr: u16,
    /// The 8-bit data bus.
    pub data: u8,
    /// Enables verbose debugging.
    pub debug: bool,
    /// CPU 'memory request' line.
    pub mem: bool,
    /// A possible pending write to the bus state during the current cycle.
    pending_write: Option<Vec<State>>,
}

impl Bus {
    /// Asserts all the given bus states.
    ///
    /// # Errors
    ///
    /// On the first failed assertion.
    #[inline]
    pub fn assert(&self, states: &[State], msg: &'static str) -> Result<()> {
        for state in states {
            match *state {
                State::Addr(addr) => ensure!(
                    self.addr == addr,
                    "want bus addr {:04X}, got {:04X} {msg}",
                    addr,
                    self.addr
                ),
                State::Data(data) => ensure!(
                    self.data == data,
                    "want bus data {:02X}, got {:02X} {msg}",
                    data,
                    self.data
                ),
                State::Mem(mem) => ensure!(
                    self.mem == mem,
                    "/MEM line {} {msg}",
                    if self.mem { "active" } else { "inactive" }
                ),
            }
        }
        Ok(())
    }

    /// Tries to set `states` on the bus at the end of this cycle.
    ///
    /// If a write is already pending, this has no effect.
    #[inline]
    pub fn defer_write(&mut self, states: Vec<State>) {
        if self.pending_write.is_none() {
            self.pending_write = Some(states);
        }
    }

    /// Applies any pending write to the bus.
    #[inline]
    pub fn reconcile(&mut self) {
        if let Some(states) = self.pending_write.take() {
            for state in states {
                match state {
                    State::Addr(addr) => self.addr = addr,
                    State::Data(data) => self.data = data,
                    State::Mem(mem) => self.mem = mem,
                }
            }
        }
    }
}

/// A desired or asserted bus state.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum State {
    Addr(u16),
    Data(u8),
    Mem(bool),
}
