use anyhow::{Result, ensure};

/// A desired or asserted bus state.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Bstate {
    /// Address bus value.
    Addr(u16),
    /// Data bus value.
    Data(u8),
    /// `/MEM` line state.
    Mem(bool),
    /// `/WR` line state.
    Write(bool),
}

/// The system bus.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
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
    pub pending_write: Option<Vec<Bstate>>,
    /// CPU 'write request' line.
    pub write: bool,
}

impl Bus {
    /// Asserts all the given bus states.
    ///
    /// # Errors
    ///
    /// On the first failed assertion.
    #[inline]
    pub fn assert(&self, states: &[Bstate], msg: impl AsRef<str>) -> Result<()> {
        let msg = msg.as_ref();
        for state in states {
            match *state {
                Bstate::Addr(addr) => ensure!(
                    self.addr == addr,
                    "want bus addr {:04X}, got {:04X} {msg}",
                    addr,
                    self.addr
                ),
                Bstate::Data(data) => ensure!(
                    self.data == data,
                    "want bus data {:02X}, got {:02X} {msg}",
                    data,
                    self.data
                ),
                Bstate::Mem(mem) => ensure!(
                    self.mem == mem,
                    "/MEM line {} {msg}",
                    if self.mem { "active" } else { "inactive" }
                ),
                Bstate::Write(wr) => ensure!(
                    self.write == wr,
                    "/WR line {} {msg}",
                    if self.write { "active" } else { "inactive" }
                ),
            }
        }
        Ok(())
    }

    /// Sets the `/MEM` line inactive.
    #[inline]
    pub fn disable_mem(&mut self) {
        self.pending_write.get_or_insert(vec![Bstate::Mem(false)]);
    }

    /// Issues a memory read request for `addr`.
    #[inline]
    pub fn mem_read(&mut self, addr: u16) {
        self.pending_write.get_or_insert(vec![
            Bstate::Addr(addr),
            Bstate::Mem(true),
            Bstate::Write(false),
        ]);
    }

    /// Issues a memory write request for `addr` with `val`.
    #[inline]
    pub fn mem_write(&mut self, addr: u16, val: u8) {
        self.pending_write.get_or_insert(vec![
            Bstate::Addr(addr),
            Bstate::Data(val),
            Bstate::Mem(true),
            Bstate::Write(true),
        ]);
    }

    /// Applies any pending write to the bus.
    #[inline]
    pub fn reconcile(&mut self) {
        if let Some(states) = self.pending_write.take() {
            for state in states {
                match state {
                    Bstate::Addr(addr) => self.addr = addr,
                    Bstate::Data(data) => self.data = data,
                    Bstate::Mem(mem) => self.mem = mem,
                    Bstate::Write(wr) => self.write = wr,
                }
            }
        }
    }

    /// Puts `data` on the data bus.
    #[inline]
    pub fn write_data(&mut self, data: u8) {
        self.pending_write.get_or_insert(vec![Bstate::Data(data)]);
    }
}

#[cfg(test)]
mod tests {
    use crate::{asm::asm, system::System};

    use super::*;

    #[test]
    fn program_executes_with_correct_bus_states() -> Result<()> {
        let mut sys = System {
            debug: true,
            ..Default::default()
        };
        sys.mem.load(
            0x0000,
            &asm("
                ld a, 0xFF
                nop
                halt"),
        )?;
        let ticks = vec![
            (
                "initial",
                &[Bstate::Addr(0x0000), Bstate::Data(0x00), Bstate::Mem(false)],
            ),
            (
                "after fetchopcode at 0x0000",
                &[Bstate::Addr(0x0000), Bstate::Data(0x00), Bstate::Mem(true)],
            ),
            (
                "after waitopcode at 0x0000",
                &[Bstate::Addr(0x0000), Bstate::Data(0x10), Bstate::Mem(true)],
            ),
            (
                "after decode ld a, 0xFF",
                &[Bstate::Addr(0x0001), Bstate::Data(0x10), Bstate::Mem(true)],
            ),
            (
                "after waitop1of1",
                &[Bstate::Addr(0x0001), Bstate::Data(0xFF), Bstate::Mem(true)],
            ),
            (
                "after readop1of1",
                &[Bstate::Addr(0x0001), Bstate::Data(0xFF), Bstate::Mem(true)],
            ),
            (
                "after execute 'ld a, 0xff'",
                &[Bstate::Addr(0x0001), Bstate::Data(0xFF), Bstate::Mem(true)],
            ),
            (
                "after fetchopcode",
                &[Bstate::Addr(0x0002), Bstate::Data(0xFF), Bstate::Mem(true)],
            ),
            (
                "after waitopcode",
                &[Bstate::Addr(0x0002), Bstate::Data(0x01), Bstate::Mem(true)],
            ),
            (
                "after decode nop",
                &[Bstate::Addr(0x0002), Bstate::Data(0x01), Bstate::Mem(false)],
            ),
            (
                "after execute 'nop'",
                &[Bstate::Addr(0x0002), Bstate::Data(0x01), Bstate::Mem(false)],
            ),
        ];

        for (msg, start_states) in ticks {
            sys.bus.assert(start_states, msg).inspect_err(|_| {
                sys.trace();
            })?;
            sys.tick();
        }
        Ok(())
    }
}
