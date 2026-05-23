#![expect(clippy::use_debug, reason = "temporary")]
use anyhow::Result;

use std::io::{Write as _, stdin, stdout};

use r8cpu::{bus::Bus, cpu::Cpu, device::Device as _, memory::Memory};

fn main() -> Result<()> {
    let mut cpu = Cpu::default();
    let mut mem = Memory::default();
    let mut bus = Bus::default();
    let mut ticks: u16 = 0;
    mem.set(0x0000, 0xFF);
    mem.set(0x0001, 0xFE);
    mem.set(0x0002, 0xFD);
    mem.set(0x0003, 0xFC);
    loop {
        bus.dirty = false;
        let mut input = String::new();
        println!(
            "Tick {:04X} Phase: {:?} Addr {:04X} Data {:02X} Mem {}",
            ticks, cpu.phase, bus.addr, bus.data, bus.mem
        );
        stdout().flush()?;
        _ = stdin().read_line(&mut input)?;
        cpu.tick(&mut bus);
        mem.tick(&mut bus);
        ticks = ticks.wrapping_add(1);
    }
}
