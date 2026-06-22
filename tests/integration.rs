// #[expect(clippy::as_conversions, reason = "Opcode is repr(u8)")]
// #[expect(clippy::tests_outside_test_module, reason = "integration test")]
// #[test]
// fn program_executes_correctly() -> Result<()> {
//     let mut sys = System::default();
//     sys.debug = true;
//     sys.mem.load(0x0000, &[LdImmByteA as u8, 0xFF, Nop as u8])?;
//     let ticks = vec![
//         (
//             "initial",
//             &[State::Addr(0x0000), State::Data(0x00), State::Mem(false)],
//         ),
//         (
//             "after fetch opcode at 0x0000",
//             &[State::Addr(0x0000), State::Data(0x00), State::Mem(true)],
//         ),
//         (
//             "after wait at 0x0000",
//             &[
//                 State::Addr(0x0000),
//                 State::Data(LdImmByteA as u8),
//                 State::Mem(true),
//             ],
//         ),
//         (
//             "after decode nop",
//             &[
//                 State::Addr(0x0000),
//                 State::Data(LdImmByteA as u8),
//                 State::Mem(true),
//             ],
//         ),
//         (
//             "after fetch operand at 0x0001",
//             &[
//                 State::Addr(0x0001),
//                 State::Data(LdImmByteA as u8),
//                 State::Mem(true),
//             ],
//         ),
//         (
//             "after wait at 0x0001",
//             &[State::Addr(0x0001), State::Data(0xFF), State::Mem(true)],
//         ),
//         (
//             "after decode at 0x0001",
//             &[State::Addr(0x0001), State::Data(0xFF), State::Mem(false)],
//         ),
//         (
//             "after execute 'ld a, 0xff'",
//             &[State::Addr(0x0001), State::Data(0xFF), State::Mem(false)],
//         ),
//         (
//             "after fetch opcode at 0x0002",
//             &[State::Addr(0x0002), State::Data(0xFF), State::Mem(true)],
//         ),
//         (
//             "after wait at 0x0002",
//             &[
//                 State::Addr(0x0002),
//                 State::Data(Nop as u8),
//                 State::Mem(true),
//             ],
//         ),
//         (
//             "after decode at 0x0002",
//             &[
//                 State::Addr(0x0002),
//                 State::Data(Nop as u8),
//                 State::Mem(false),
//             ],
//         ),
//         (
//             "after execute 'nop'",
//             &[
//                 State::Addr(0x0002),
//                 State::Data(Nop as u8),
//                 State::Mem(false),
//             ],
//         ),
//     ];

//     for (msg, start_states) in ticks {
//         sys.bus.assert(start_states, msg).inspect_err(|_| {
//             sys.trace();
//         })?;
//         sys.tick();
//     }
//     Ok(())
// }
