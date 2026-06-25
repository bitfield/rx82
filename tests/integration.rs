use anyhow::Result;

use rx82::{
    instructions::InstructionKind::*,
    regs::Reg8::*,
    system::{State, System},
};

#[expect(clippy::tests_outside_test_module, reason = "integration test")]
#[test]
fn program_executes_correctly() -> Result<()> {
    let mut sys = System::default();
    sys.debug = true;
    sys.mem.load(
        0x0000,
        &[
            u8::from(LoadRegImm8(A)),
            0xFF,          // ld a, 0xFF
            u8::from(Nop), // nop
        ],
    )?;
    let ticks = vec![
        (
            "initial",
            &[State::Addr(0x0000), State::Data(0x00), State::Mem(false)],
        ),
        (
            "after fetch opcode at 0x0000",
            &[State::Addr(0x0000), State::Data(0x00), State::Mem(true)],
        ),
        (
            "after wait at 0x0000",
            &[State::Addr(0x0000), State::Data(0x10), State::Mem(true)],
        ),
        (
            "after decode nop",
            &[State::Addr(0x0000), State::Data(0x10), State::Mem(true)],
        ),
        (
            "after fetch operand at 0x0001",
            &[State::Addr(0x0001), State::Data(0x10), State::Mem(true)],
        ),
        (
            "after wait at 0x0001",
            &[State::Addr(0x0001), State::Data(0xFF), State::Mem(true)],
        ),
        (
            "after decode at 0x0001",
            &[State::Addr(0x0001), State::Data(0xFF), State::Mem(false)],
        ),
        (
            "after execute 'ld a, 0xff'",
            &[State::Addr(0x0001), State::Data(0xFF), State::Mem(false)],
        ),
        (
            "after fetch opcode at 0x0002",
            &[State::Addr(0x0002), State::Data(0xFF), State::Mem(true)],
        ),
        (
            "after wait at 0x0002",
            &[State::Addr(0x0002), State::Data(0x01), State::Mem(true)],
        ),
        (
            "after decode at 0x0002",
            &[State::Addr(0x0002), State::Data(0x01), State::Mem(false)],
        ),
        (
            "after execute 'nop'",
            &[State::Addr(0x0002), State::Data(0x01), State::Mem(false)],
        ),
    ];

    for (msg, start_states) in ticks {
        sys.bus.assert(start_states, msg).inspect_err(|_| {
            sys.trace();
        })?;
        sys.tick();
    }
    Ok(())
}
