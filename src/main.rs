use anyhow::Result;

use r8cpu::system::System;

fn main() -> Result<()> {
    let mut sys = System::default();
    // write some interesting junk to memory,
    // just so we can see it being fetched
    sys.mem.set(0x0000, 0xFF);
    sys.mem.set(0x0001, 0xFE);
    sys.mem.set(0x0002, 0xFD);
    sys.mem.set(0x0003, 0xFC);
    loop {
        sys.debug_print()?;
        sys.tick();
    }
}
