use std::io::{Write as _, stdin, stdout};

use anyhow::Result;

use crate::{
    cpu::{Phase, Target},
    system::System,
};

/// The interactive CLI system monitor.
#[non_exhaustive]
#[derive(Default)]
pub struct Monitor {
    pub debug: bool,
    pub sys: System,
}

impl Monitor {
    /// Runs the monitor interactively until the user quits.
    ///
    /// # Errors
    ///
    /// If reading the user's command input fails.
    #[inline]
    pub fn run(&mut self) -> Result<()> {
        self.sys.debug = self.debug;
        loop {
            if self.sys.cpu.halt {
                println!("halted");
                self.debug = true;
            }
            if self.debug
                && self.sys.cpu.phase == Phase::Fetch
                && self.sys.cpu.target == Target::Opcode
            {
                self.sys.debug_cpu();
                wait_for_newline()?;
            }
            self.sys.cpu.halt = false;
            self.sys.tick();
        }
    }
}

/// Prompts and reads the user's next command.
#[expect(clippy::single_call_fn, reason = "readability")]
fn wait_for_newline() -> Result<()> {
    let mut input = String::new();
    print!("> ");
    stdout().flush()?;
    _ = stdin().read_line(&mut input)?;
    Ok(())
}
