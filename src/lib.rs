#![cfg_attr(doc, doc = include_str!("../README.md"))]
pub mod bus;
pub mod cpu;
pub mod device;
pub mod instructions;
pub mod memory;
pub mod regs;

#[cfg(test)]
mod tests;
