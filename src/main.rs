use std::io::{Write as _, stdin, stdout};

use anyhow::Result;

use rx82::{
    cpu::Phase::FetchOpcode,
    instructions::{LDA_N, NOP},
    system::System,
};

fn main() -> Result<()> {
    let mut sys = System::default();
    sys.debug = true;
    sys.mem
        .load(0x0000, &[NOP, LDA_N, 0xFF, LDA_N, 0xFE, LDA_N, 0xFD])?;
    loop {
        if sys.cpu.phase == FetchOpcode {
            sys.debug_cpu();
            wait_for_newline()?;
        }
        sys.tick()?;
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
