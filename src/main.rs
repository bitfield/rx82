use anyhow::Result;

use r8cpu::system::System;

#[expect(clippy::expect_used, reason = "temporary")]
fn main() -> Result<()> {
    let mut sys = System::default();
    // write some interesting junk to memory,
    // just so we can see it being fetched
    sys.mem
        .load(0x0000, &[0xFF, 0xFE, 0xFD, 0xFC])
        .expect("load failed");
    loop {
        sys.debug_print()?;
        sys.tick();
    }
}
