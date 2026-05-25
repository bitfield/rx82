#![cfg_attr(doc, doc = include_str!("../README.md"))]
pub mod bus;
pub mod clock;
pub mod cpu;
pub mod device;
pub mod instructions;
pub mod memory;
pub mod regs;
pub mod system;

#[cfg(test)]
mod tests;
