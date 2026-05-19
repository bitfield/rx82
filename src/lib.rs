#![cfg_attr(doc, doc = include_str!("../README.md"))]

use std::{collections::HashMap, sync::LazyLock};

pub trait Device {
    fn step(&mut self, mem: &mut Memory);
}

#[non_exhaustive]
#[derive(Debug)]
pub struct Memory(Vec<u8>);

impl Default for Memory {
    fn default() -> Self {
        Self(vec![0; 0xFFFF])
    }
}

impl Memory {
    #[must_use]
    pub fn get(&self, addr: u16) -> u8 {
        self.0.get(usize::from(addr)).copied().unwrap_or_default()
    }

    pub fn set(&mut self, addr: u16, val: u8) {
        if let Some(loc) = self.0.get_mut(usize::from(addr)) {
            *loc = val;
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Cpu {
    pub pc: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub g: u8,
    pub h: u8,
    pub p: u8,
    pub s: u16,
}

impl Device for Cpu {
    fn step(&mut self, mem: &mut Memory) {
        let opcode = mem.get(self.pc);
        self.pc = self.pc.wrapping_add(1);
        println!("opcode {opcode}");
        if let Some(ins) = INSTRUCTIONS.get(&opcode) {
            (ins.execute)(self, mem);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub name: &'static str,
    pub execute: fn(&mut Cpu, &mut Memory),
    pub test: fn(&mut Cpu, &mut Memory),
}

static INSTRUCTIONS: LazyLock<HashMap<u8, &Instruction>> = LazyLock::new(|| {
    HashMap::from([(
        0x01,
        &Instruction {
            name: "ld a, N",
            execute: |cpu, mem| {
                cpu.a = mem.get(cpu.pc);
                cpu.pc = cpu.pc.wrapping_add(1);
            },
            test: |_, _| {},
        },
    )])
});
