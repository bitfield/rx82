use crate::{bus::Bus, cpu::Cpu, device::Device as _, memory::Memory};

#[test]
fn bus_has_correct_states() {
    let mut cpu = Cpu::default();
    let mut mem = Memory::default();
    let mut bus = Bus::default();
    mem.set(0x0000, 0xFF); // junk
    cpu.tick(&mut bus); // fetch opcode
    assert_eq!(bus.addr, 0x0000, "wrong addr after cpu fetch cycle");
    assert_eq!(bus.data, 0x00, "wrong data after cpu fetch cycle");
    assert!(bus.dirty, "bus not dirty after cpu fetch cycle");
    bus.dirty = false; // next system cycle
    cpu.tick(&mut bus); // memwait
    assert_eq!(bus.data, 0x00, "wrong data after cpu memwait");
    mem.tick(&mut bus);
    assert_eq!(bus.addr, 0x0000, "wrong addr after mem cycle");
    assert_eq!(bus.data, 0xFF, "wrong data after mem cycle");
    assert!(bus.dirty, "bus not dirty after mem cycle");
    bus.dirty = false; // next system cycle
    cpu.tick(&mut bus); // execute unknown opcode (= noop)
    assert_eq!(bus.addr, 0x0000, "fetching too early");
    assert_eq!(bus.data, 0xFF, "bus data changed");
    mem.tick(&mut bus);
    assert_eq!(bus.data, 0xFF, "memory wrote to bus");
    bus.dirty = false; // next system cycle
    cpu.tick(&mut bus); // fetch opcode
    assert!(bus.dirty, "bus not written on fetch cycle");
    assert_eq!(bus.addr, 0x0001, "not fetching next address after execute");
    assert_eq!(bus.data, 0xFF, "bus data changed");
}
