use anyhow::Result;

use std::io::{Write as _, stdin, stdout};

use r8cpu::{cpu::Cpu, device::Device as _, memory::Memory};

#[expect(clippy::use_debug, reason = "still debugging")]
fn main() -> Result<()> {
    let mut cpu = Cpu::default();
    let mut mem = Memory::default();
    mem.set(0x0000, 0x10);
    mem.set(0x0001, 0xFF);
    let mut input = String::new();
    loop {
        println!("{cpu:?}");
        print!("> ");
        stdout().flush()?;
        let n = stdin().read_line(&mut input)?;
        if n == 0 {
            break;
        }
        cpu.step(&mut mem);
    }
    Ok(())
}
