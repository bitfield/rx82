use anyhow::Result;

use rx82::{
    bus::State,
    instructions::{LDA_N, NOP},
    system::System,
};

#[expect(clippy::tests_outside_test_module, reason = "integration test")]
#[test]
fn program_executes_correctly() -> Result<()> {
    let mut sys = System::default();
    sys.debug = true;
    sys.mem.load(
        0x0000,
        &[
            LDA_N, 0xFF, // 0000 ld a, 0xFF
            NOP,  //        0002 nop
        ],
    )?;
    let ticks = vec![
        (
            "initial",
            &[State::Addr(0x0000), State::Data(0x00), State::Mem(false)],
        ),
        (
            "after FetchOpcode 0x0000",
            &[State::Addr(0x0000), State::Data(0x00), State::Mem(true)],
        ),
        (
            "after WaitOpcode 0x0000",
            &[State::Addr(0x0000), State::Data(LDA_N), State::Mem(true)],
        ),
        (
            "after Decode nop",
            &[State::Addr(0x0000), State::Data(LDA_N), State::Mem(true)],
        ),
        (
            "after FetchOperand 0x0001",
            &[State::Addr(0x0001), State::Data(LDA_N), State::Mem(true)],
        ),
        (
            "after WaitOperand 0x0001",
            &[State::Addr(0x0001), State::Data(0xFF), State::Mem(true)],
        ),
        (
            "after ReadOperand 0x0001",
            &[State::Addr(0x0001), State::Data(0xFF), State::Mem(false)],
        ),
        (
            "after Execute ld a, 0xff",
            &[State::Addr(0x0001), State::Data(0xFF), State::Mem(false)],
        ),
        (
            "after FetchOpcode 0x0002",
            &[State::Addr(0x0002), State::Data(0xFF), State::Mem(true)],
        ),
        (
            "after WaitOpcode 0x0000",
            &[State::Addr(0x0002), State::Data(NOP), State::Mem(true)],
        ),
        (
            "after Decode nop",
            &[State::Addr(0x0002), State::Data(NOP), State::Mem(false)],
        ),
        (
            "after Execute nop",
            &[State::Addr(0x0002), State::Data(NOP), State::Mem(false)],
        ),
    ];

    for (msg, start_states) in ticks {
        sys.bus.assert(start_states, msg).inspect_err(|_| {
            sys.trace();
        })?;
        sys.tick()?;
    }
    Ok(())
}
