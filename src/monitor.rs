use std::{
    fs, io::{Write as _, stdin, stdout}, path::Path
};

use anyhow::Result;

use crate::{cpu::Phase::FetchOpcode, system::System};

#[non_exhaustive]
#[derive(Default)]
pub struct Monitor(pub System);

impl Monitor {
    #[expect(clippy::impl_trait_in_params, reason = "readability")]
    /// Loads the binary file at `path` into memory at address `addr`.
    ///
    /// # Errors
    ///
    /// If loading the file fails or if the data cannot be loaded at the requested
    /// address (for example, if it would go out of bounds).
    #[inline]
    pub fn load_bin(&mut self, addr: u16, path: impl AsRef<Path>) -> Result<()> {
        let data = fs::read(path)?;
        self.0.mem.load(addr, &data)?;
        Ok(())
    }

    /// Runs the monitor interactively until the user quits.
    ///
    /// # Errors
    ///
    /// If reading the user's command input fails.
    #[inline]
    pub fn run(&mut self) -> Result<()> {
        self.0.debug = true;
        loop {
            if self.0.cpu.phase == FetchOpcode {
                self.0.debug_cpu();
                wait_for_newline()?;
            }
            self.0.tick()?;
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
    fn load_bin_fn_loads_binary_file() {
        let mut mon = Monitor::default();
        mon.load_bin(0x000, "tests/data/test.bin").unwrap();
        assert_eq!(
            &mon.0.mem.0[0..=5],
            &[LDA_N, 0xFF, LDA_N, 0xFE, LDA_N, 0xFD],
            "wrong data"
        );
    }
}
