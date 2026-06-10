use std::io::{Write as _, stdin, stdout};

use anyhow::Result;

use crate::{cpu::Phase::FetchOpcode, system::System};

#[non_exhaustive]
#[derive(Default)]
pub struct Monitor {
    pub debug: bool,
    pub sys: System,
}

impl Monitor {
    /// Loads the binary file at `path` into memory at address `addr`.
    ///
    /// # Errors
    ///
    /// If loading the file fails or if the data cannot be loaded at the requested
    /// address (for example, if it would go out of bounds).
    #[inline]
    pub fn load(&mut self, addr: u16, data: &[u8]) -> Result<()> {
        self.sys.mem.load(addr, data)
    }

    /// Runs the monitor interactively until the user quits.
    ///
    /// # Errors
    ///
    /// If reading the user's command input fails.
    #[inline]
    pub fn run(&mut self) -> Result<()> {
        self.sys.debug = self.debug;
        loop {
            if self.debug && self.sys.cpu.phase == FetchOpcode {
                self.sys.debug_cpu();
                wait_for_newline()?;
            }
            self.sys.tick()?;
        }
    }
}

#[expect(clippy::single_call_fn, reason = "readability")]
fn wait_for_newline() -> Result<()> {
    let mut input = String::new();
    print!("> ");
    stdout().flush()?;
    _ = stdin().read_line(&mut input)?;
    Ok(())
}

#[expect(clippy::unwrap_used, reason = "tests")]
#[cfg(test)]
mod tests {
    use crate::instructions::LDA_N;

    use super::*;

    #[expect(clippy::indexing_slicing, reason = "must succeed for test to pass")]
    #[test]
    fn load_loads_data() {
        let mut mon = Monitor::default();
        let data = &[LDA_N, 0xFF, LDA_N, 0xFE, LDA_N, 0xFD];
        mon.load(0x000, data).unwrap();
        assert_eq!(&mon.sys.mem.0[0..=5], data, "wrong data");
    }
}
