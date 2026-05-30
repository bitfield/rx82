use std::io::{Write as _, stdin, stdout};

use anyhow::Result;

use rx82::{phase::Phase::FetchOpcode, regs::Reg8::A, system::System};

fn main() -> Result<()> {
    let mut sys = System::default();
    // write some interesting junk to memory,
    // just so we can see it being fetched
    sys.mem.load(0x0000, &[0xFF, 0xFE, 0xFD, 0xFC])?;
    loop {
        if sys.cpu.phase == FetchOpcode {
            println!("  PC  A NXT");
            println!(
                "{:04X} {:02X}  {:02X}",
                sys.cpu.pc,
                sys.cpu.regs.get8(A),
                sys.mem.get(sys.cpu.pc),
            );
            let mut input = String::new();
            print!("> ");
            stdout().flush()?;
            _ = stdin().read_line(&mut input)?;
        }
        sys.tick()?;
    }
}
