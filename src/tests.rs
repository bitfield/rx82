use crate::{bus::State, system::System};

#[test]
fn bus_has_correct_states() {
    let mut sys = System::default();
    sys.mem.set(0x0000, 0xFF); // junk

    // Tick 0: CPU issues fetch with PC=0000
    sys.tick();
    sys.bus.assert(
        &[State::Addr(0x0000), State::Data(0x00), State::Mem(true)],
        "after fetch 0x0000",
    );

    // Tick 1: mem reads 0xFF, writes to bus
    sys.tick();
    sys.bus.assert(
        &[State::Addr(0x0000), State::Data(0xFF), State::Mem(true)],
        "after memread 0x0000",
    );

    // Tick 2: CPU executes junk opcode (ignored)
    sys.tick();
    sys.bus.assert(
        &[State::Addr(0x0000), State::Data(0xFF), State::Mem(false)],
        "after execute noop",
    );

    // Tick 3: CPU issues fetch with PC=0001
    sys.tick();
    sys.bus.assert(
        &[State::Addr(0x0001), State::Data(0xFF), State::Mem(true)],
        "after fetch 0x0001",
    );

    // Tick 4: mem reads 0x00, writes to bus
    sys.tick();
    sys.bus.assert(
        &[State::Addr(0x0001), State::Data(0x00), State::Mem(true)],
        "after memread 0x0001",
    );
}
