use std::io::{Write as _, stdin, stdout};

use anyhow::Result;

use rx82::{
    instructions::{LDA_N, NOP},
    system::System,
};

fn main() -> Result<()> {
    let mut sys = System::default();
    sys.debug = true;
    sys.mem
        .load(0x0000, &[NOP, LDA_N, 0xFF, LDA_N, 0xFE, LDA_N, 0xFD])?;
    sys.debug_cpu();
    wait_for_newline()?;
    loop {
        sys.tick()?;
        if sys.cpu.instruction_complete {
            sys.debug_cpu();
            wait_for_newline()?;
        }
    }
}

fn wait_for_newline() -> Result<()> {
    let mut input = String::new();
    print!("> ");
    stdout().flush()?;
    _ = stdin().read_line(&mut input)?;
    Ok(())
}
