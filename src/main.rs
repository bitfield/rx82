use anyhow::Result;

use std::io::{Write as _, stdin, stdout};

use r8cpu::{cpu::Cpu, device::Device as _, memory::Memory, regs::Reg8::A};

fn main() -> Result<()> {
    let mut cpu = Cpu::default();
    let mut mem = Memory::default();
    mem.set(0x0000, 0x10); // ld a, 0xFF
    mem.set(0x0001, 0xFF);
    let mut input = String::new();
    println!("  PC  A OP");
    loop {
        print!(
            "\r{:04X} {:02X} {:02X} > ",
            cpu.pc,
            cpu.regs.get8(A),
            mem.get(cpu.pc)
        );
        stdout().flush()?;
        let n = stdin().read_line(&mut input)?;
        if n == 0 {
            break;
        }
        cpu.step(&mut mem);
    }
    Ok(())
}
