#![cfg_attr(doc, doc = include_str!("../README.md"))]

pub mod asm;

pub use asm::{
    Disassembler, as_hex, assemble, assemble_source_file, assemble_with_debug, disassemble,
};
