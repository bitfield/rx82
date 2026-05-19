use anyhow::Result;

use std::io::{Write, stdin, stdout};

use r8cpu::{Cpu, Device, Memory};

fn main() -> Result<()> {
    let mut cpu = Cpu::default();
    let mut mem = Memory::default();
    mem.set(0x0000, 0x01);
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
