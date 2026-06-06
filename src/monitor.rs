use std::io::{Write as _, stdin, stdout};

use anyhow::Result;

use crate::{cpu::Phase::FetchOpcode, system::System};

#[non_exhaustive]
#[derive(Default)]
pub struct Monitor(pub System);

impl Monitor {
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
