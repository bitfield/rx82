#[non_exhaustive]
#[derive(Debug, Default)]
#[expect(clippy::partial_pub_fields, reason = "dirty is internal")]
pub struct Bus {
    pub addr: u16,
    pub data: u8,
    dirty: bool,
    pub mem: bool,
    pending_write: Vec<State>,
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
        if !self.dirty {
            self.pending_write = states;
            self.dirty = true;
        }
    }

    #[inline]
    pub fn reconcile(&mut self) {
        for state in &self.pending_write {
            match *state {
                State::Addr(addr) => self.addr = addr,
                State::Data(data) => self.data = data,
                State::Mem(mem) => self.mem = mem,
            }
        }
        self.dirty = false;
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum State {
    Addr(u16),
    Data(u8),
    Mem(bool),
}
