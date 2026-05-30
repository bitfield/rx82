use std::io::{Write as _, stdin, stdout};

use anyhow::Result;

use rx82::{
    asm::disassemble,
    instructions::{LDA_N, NOP},
    phase::Phase::FetchOpcode,
    regs::Reg8::A,
    system::System,
};

fn main() -> Result<()> {
    let mut sys = System::default();
    // write some interesting junk to memory,
    // just so we can see it being fetched
    sys.mem.load(0x0000, &[NOP, LDA_N, 0xFF, LDA_N, 0xFE])?;
    loop {
        if sys.cpu.phase == FetchOpcode {
            println!("  PC  A NEXT");
            println!(
                "{:04X} {:02X} {}",
                sys.cpu.pc,
                sys.cpu.regs.get8(A),
                disassemble(sys.mem.slice_from(sys.cpu.pc)?),
            );
            let mut input = String::new();
            print!("> ");
            stdout().flush()?;
            _ = stdin().read_line(&mut input)?;
        }
        sys.tick()?;
    }
}
