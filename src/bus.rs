#[non_exhaustive]
#[derive(Debug, Default)]
#[expect(clippy::partial_pub_fields, reason = "pending_write is internal")]
pub struct Bus {
    pub addr: u16,
    pub data: u8,
    pub debug: bool,
    pub mem: bool,
    pending_write: Option<Vec<State>>,
}

impl Bus {
    /// Asserts all the given bus states.
    ///
    /// # Panics
    ///
    /// On the first failed assertion.
    #[inline]
    pub fn assert(&self, states: &[State], msg: &'static str) {
        for state in states {
            match *state {
                State::Addr(addr) => assert_eq!(
                    self.addr, addr,
                    "want bus addr {:04X}, got {:04X} {msg}",
                    addr, self.addr
                ),
                State::Data(data) => assert_eq!(
                    self.data, data,
                    "want bus data {:02X}, got {:02X} {msg}",
                    data, self.data
                ),
                State::Mem(mem) => assert_eq!(
                    self.mem,
                    mem,
                    "mem line {} {msg}",
                    if self.mem { "active" } else { "inactive" }
                ),
            }
        }
    }

    #[inline]
    pub fn defer_write(&mut self, states: Vec<State>) {
        if self.pending_write.is_none() {
            self.pending_write = Some(states);
        }
    }

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

#[non_exhaustive]
#[derive(Debug)]
pub enum State {
    Addr(u16),
    Data(u8),
    Mem(bool),
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{bus::State, system::System};

    #[test]
    fn bus_has_correct_states() -> Result<()> {
        let mut sys = System::default();
        sys.mem.load(0x0000, &[0xFF])?; // junk

        // Tick 0: CPU issues fetch with PC=0000
        sys.tick()?;
        sys.bus.assert(
            &[State::Addr(0x0000), State::Data(0x00), State::Mem(true)],
            "after fetch 0x0000",
        );

        // Tick 1: mem reads 0xFF, writes to bus
        sys.tick()?;
        sys.bus.assert(
            &[State::Addr(0x0000), State::Data(0xFF), State::Mem(true)],
            "after memread 0x0000",
        );

        // Tick 2: CPU executes junk opcode (ignored)
        sys.tick()?;
        sys.bus.assert(
            &[State::Addr(0x0000), State::Data(0xFF), State::Mem(false)],
            "after execute noop",
        );

        // Tick 3: CPU issues fetch with PC=0001
        sys.tick()?;
        sys.bus.assert(
            &[State::Addr(0x0001), State::Data(0xFF), State::Mem(true)],
            "after fetch 0x0001",
        );

        // Tick 4: mem reads 0x00, writes to bus
        sys.tick()?;
        sys.bus.assert(
            &[State::Addr(0x0001), State::Data(0x00), State::Mem(true)],
            "after memread 0x0001",
        );
        Ok(())
    }
}
