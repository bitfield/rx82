[![Crate](https://img.shields.io/crates/v/rx82.svg)](https://crates.io/crates/rx82)
[![Docs](https://docs.rs/rx82/badge.svg)](https://docs.rs/rx82)
![CI](https://github.com/bitfield/rx82/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/bitfield/rx82/actions/workflows/audit.yml/badge.svg)
![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

An emulator for the RX82 fantasy retro computer system, including the R8 8-bit CPU.

> ADRIC: _What do these numbers and letters mean?_\
> DOCTOR: _It's an early version. Instructions have to be punched in by machine code._\
> ADRIC: _Oh, how boring._\
> DOCTOR: _**Boring?**_\
—Doctor Who, _Logopolis_

![](img/RX82.jpg)

# Installation

```sh
cargo install rx82
```

# About

This is an emulator for the RX82 architecture, an imagined home computer system similar to those of the early 1980s, such as the Sinclair ZX81 and Spectrum, the BBC Micro, or the Commodore 64.

The RX82's design is intended not only to evoke fond memories in those of a certain age, but also to help teach the fundamentals of computer systems architecture and computer engineering. It's simpler than historic systems such as the ZX81, because no cost or design compromises are required, but also realistic enough to be useful for learning purposes.

Its central processor is the R8, a fan-fiction CPU design comparable to the Zilog Z80 or the MOS 6502, but again, somewhat simplified for educational purposes.

This crate provides a reference implementation of the RX82 and R8 architectures, and an assembler / disassembler for use with R8 assembly language programs. However, it is intended to be modular, so that you can pick and choose components to build your own systems.

For example, you could use the R8 CPU as part of your own emulator that replaces the RX82 system with something else. Equally, you could use the RX82 system components but replace the CPU with a design of your own, or an emulated real machine such as a 6502.

# Usage

## Assembling R8 source files

Prepare your program in a text file (see _R8 Assembly Language_ below), and run:

```sh
rx82 asm my_prog.asm
```

If the program assembles correctly, this will produce a `my_prog.bin` file you can run with the monitor.

To assemble with verbose debugging (probably only of interest to RX82 developers), use the `--debug` switch:

```sh
rx82 asm --debug my_prog.asm
```


## Starting the monitor

To start the monitor in debug (single-step) mode:

```sh
rx82 mon
```

You can also optionally load and run a binary file (such as one produced by the assembler, for example):

```sh
rx82 mon my_prog.bin
```

The program will run until a `halt` instruction is reached.

To run the program in single-step mode, use the `--debug` switch:

```sh
rx82 mon --debug my_prog.bin
```

## Using the monitor

In single-step mode, the monitor displays the current CPU registers and the next instruction in memory, then prompts for a command:

```txt
  PC  A  B  C  D  E  F  G  H NEXT
0003 DE AD 00 00 00 00 00 00 ld cd, 0xBEEF
>
```

To execute the next CPU instruction, press Enter, or press Ctrl-C to exit.

# RX82 user's manual

## The RX82 architecture

The RX82 is a single-board computer with one R8 CPU clocked at 4Mhz, 64KiB of static RAM, an 8-bit data bus, and a 16-bit address bus.

## The R8 CPU

The R8 is an 8-bit CPU with some 16-bit features. It has eight 8-bit registers: A, B, C, D, E, F, G, and H. Similar to the Z80, these can also be addressed as four 16-bit register pairs: AB, CD, EF, and GH.

The 16-bit address bus allows the R8 to address up to 64KiB of memory.

## R8 assembly language

The input format recognised by the R8 assembler is very similar to that of most Z80 or 6502 assemblers. Here's a simple example program:

```asm
    ld a, 0xDE
    ld b, 0xAD
    ld cd, 0xBEEF
    ld ef, 0xCAFE
    ld gh, 0xBABE
    ld 0xFFFF, a
    halt
```

Whitespace is ignored, and only `0x`-prefixed hexadecimal numbers are recognised as literals.

## Opcodes

| HI/LO |  00      |  01      |  02      |  03      |  04      |  05      |  06      |  07      |  08       |  09       |  0A       |  0B       |  0C   |  0D   |  0E   |  0F   |
| :---: | :---:    | :---:    | :---:    | :---:    | :---:    | :---:    | :---:    | :---:    | :---:     | :---:     | :---:     | :---:     | :---: | :---: | :---: | :---: |
| 00    | halt     | nop      | --       | --       | --       | --       | --       | --       | --        | --        | --        | --        | --    | --    | --    | --    |
| 01    | ld a, N  | ld b, N  | ld c, N  | ld d, N  | ld e, N  | ld f, N  | ld g, N  | ld h, N  | ld ab, NN | ld cd, NN | ld ef, NN | ld gh, NN | --    | --    | --    | --    |
| 02    | ld NN, a | ld NN, b | ld NN, c | ld NN, d | ld NN, e | ld NN, f | ld NN, g | ld NN, h | --        | --        | --        | --        | --    | --    | --    | --    |

# Changelog

* **0.2.0** — monitor improvements, add `halt` instruction, add assembler
* **0.1.0** — first release
