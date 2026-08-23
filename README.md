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

```txt
(C) 1982 RX Computers Ltd.
Ready.
```

You can also optionally load and run a binary file (such as one produced by the assembler, for example):

```sh
rx82 mon my_prog.bin
```

## Using the monitor

The monitor displays the current CPU registers and the next instruction in memory, then prompts for a command. Type `H` for help:

```txt
RMON v1.0 (C) 1977 Solid State Technologies, Inc.
  PC   SP  A  B  C  D  E  F  G  H ZC | NEXT
C022 BFFF 02 00 BF FF 00 00 00 00 00 | halt
> h
Commands:
G [<address>] = Go (run till halted)
H             = Help
M [<address>] = Memory dump
S [<address>] = Single step
Q             = Quit
Enter         = Repeat last command
>
```

To dump memory, use the `M` command. This will print a block of memory starting at the current value of `pc`:

```txt
> m
0000: 10 06 19 FF FF 49 B2 FD 40 B2 F7 00 00 00 00 00
0010: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
...
```

Press Enter to dump the next block:

```txt
>
0080: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
0090: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
...
```

To dump memory from a specific address, enter the address in hex:

```txt
> m fff0
FFF0: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 C0
0000: 10 06 19 FF FF 49 B2 FD 40 B2 F7 00 00 00 00 00
...
```

## Disassembling R8 binary files

Run:

```sh
rx82 dis my_prog.bin
```

This will print the disassembled listing.

# RX82 user's manual

## The RX82 architecture

The RX82 is a single-board computer with one R8 CPU clocked at 4Mhz, 64KiB of static RAM, an 8-bit data bus, and a 16-bit address bus.

## Memory map

| Address | Contents |
| :--- | :---    |
| 0x0000 | Trap/interrupt table |
| 0x0080 | System data area |
| 0x0100 | User RAM |
| 0xC000 | ROM |
| 0xFF00 | System I/O area |
| 0xFFFE | Reset vector |

## Boot process

At power on, the CPU loads the reset vector at 0xFFFE, which in the RX82 system holds the ROM entry point, 0xC000. Execution begins here and a simple RAM test is performed to find the highest writable address in memory. The stack pointer is initialised to this address.

The trap table is initialised, and all undefined traps are vectored to a single 'undefined trap' handler.

Finally, the interactive monitor is invoked.

## OS traps

The following general-purpose traps are defined:

| Code | Name | Purpose | Inputs |
| :--- | :--- | :--- | :--- |
| 0x20 | PUTCHAR | Print character to terminal | A = ASCII code of character |

# R8 technical manual

The R8 is a little-endian CPU with an 8-bit data bus, a 16-bit address bus, and a 16-bit ALU.

## Registers

The CPU has eight 8-bit registers: `a`, `b`, `c`, `d`, `e`, `f`, `g`, and `h`. As with the Z80, these can also be addressed as 16-bit register pairs: `ab`, `cd`, `ef`, and `gh`.

The 16-bit address bus allows the R8 to address up to 64KiB of memory, and the program counter register `pc` holds the 16-bit address of the next memory location to fetch from.

## Flags

The processor status register `ps` contains the following flags:

* **Carry** (bit 0) — After addition, this is the carry result. After subtraction or comparison, this flag is set if no borrow occurred (that is, for X - Y, if X >= Y). Increment and decrement instructions do not affect the carry flag.
* **Zero** (bit 1) — After instructions with a value result, this flag is set if the result is zero, or cleared otherwise.

## Opcodes

| HI/LO | -0 | -1 | -2 | -3 | -4 | -5 | -6 | -7 | -8 | -9 | -A | -B | -C | -D | -E | -F |
| :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| 0- | halt | nop | | sec | clc | | | | ret | rti | | | | | | |
| 1- | ld a, N | ld b, N | ld c, N | ld d, N | ld e, N | ld f, N | ld g, N | ld h, N | ld ab, NN | ld cd, NN | ld ef, NN | ld gh, NN | ld sp, NN | | | |
| 2- | ld NN, a | ld NN, b | ld NN, c | ld NN, d | ld NN, e | ld NN, f | ld NN, g | ld NN, h | | | | | | ld R, (RR) | ld (RR), R | ld R, R |
| 3- | inc a | inc b | inc c | inc d | inc e | inc f | inc g | inc h | inc ab | inc cd | inc ef | inc gh | inc sp | inc (RR) | inc (NN) | |
| 4- | dec a | dec b | dec c | dec d | dec e | dec f | dec g | dec h | dec ab | dec cd | dec ef | dec gh | dec sp | dec (RR) | dec (NN) | |
| 5- | add a, N | add b, N | add c, N | add d, N | add e, N | add f, N | add g, N | add h, N | | | | | | | | |
| 6- | | | | | | | | | | | | | | | | |
| 7- | cmp a, N | cmp b, N | cmp c, N | cmp d, N | cmp e, N | cmp f, N | cmp g, N | cmp h, N | cmp ab, N | cmp cd, N | cmp ef, N | cmp gh, N | | | | |
| 8- | and a, N | and b, N | and c, N | and d, N | and e, N | and f, N | and g, N | and h, N | | | | | | | | |
| 9- | | | | | | | | | | | | | | | | |
| A- | | | | | | | | | lsr a, S | lsr b, S | lsr c, S | lsr d, S | lsr e, S | lsr f, S | lsr g, S | lsr h, S |
| B- | | | | | | | | | | | | | | | | |
| C- | | | | | | | | | | | | | | | | |
| D- | push a | push b | push c | push d | push e | push f | push g | push h | push ab | push cd | push ef | push gh | | | | |
| E- | pop a | pop b | pop c | pop d | pop e | pop f | pop g | pop h | pop ab | pop cd | pop ef | pop gh | | | | |
| F- | bra D | beq D | bne D | bcs D | bcc D | | | | call NN | trap T | | | | | | |

Key:

* `D`: signed byte branch displacement (-128 to +127)
* `N`: immediate value (`NN` = 16-bit values only)
* `R`: register (`RR` = 16-bit registers only)
* `S`: unsigned byte shift count (0x01-0x08)
* `T`: trap code byte (0x00-0x3F)

## Stack

The hardware stack is governed by the stack pointer register `sp`, which should be initialised to a suitable address in RAM. The stack grows downwards unboundedly. `sp` always points to the next location where a value will be pushed: that is, an address one byte lower than the address of the current top-of-stack value.

16-bit registers and return addresses are pushed in little-endian order: that is, the low byte is pushed first, followed by the high byte.

## Reset

On reset, all registers and flags are zeroed and cleared, and `pc` is initialised from the little-endian reset vector at 0xFFFE.

## Traps

Traps are the R8's way of handling exceptions, interrupts, and user-defined toolbox routines. When a trap occurs, the following stack frame will be generated:

| Address | Data | 
| :--- | :--- |
| (`sp`+1) | Trap code (0x00-0x3F) |
| (`sp`+2) | Return address (high byte) |
| (`sp`+3) | Return address (low byte) |

### Exceptions

Trap codes 0x00-0x0F are reserved for CPU exceptions. The following exception traps are defined:

| Code | Name | Reason | 
| :--- | :--- | :--- | 
| 0x00 | ILLEGAL | Illegal instruction |

### Interrupts

Trap codes 0x10-0x1F are reserved for interrupts.

### User-defined traps

Trap codes 0x20-0x3F may be used for user-defined traps. See the operating system documentation for details of any traps provided by the OS.

The `trap` instruction will cause the CPU to trap with the specified code:

```asm
    trap 0x20
```

### Handlers

Each trap code is associated with a (little-endian) handler vector address in the trap table, spanning from address 0x0000-0x007F. The handler vector for trap `T` is at address 2 * `T`.

For example, to install a handler for trap 0x20, write its vector to address 0x0040:

```asm
; set up the 'putchar' handler
    ld cd, PUTCHAR
    ld 0x0040, d  ; vector 0x20
    ld 0x0041, c
```

To return from a handler routine, use the `rti` instruction, which will clear the trap stack frame and return to a point immediately after the trapping instruction.

# Programming the R8

The input format recognised by the R8 assembler is very similar to that of most Z80 or 6502 assemblers. Here's a simple example:

```asm
; RAM test
    ld  cd, 0xFFFD  ; top of possible RAM
    ld  a, 0x02

RAM_FILL:
    ld  (cd), a     ; write 0x02 to each location
    dec cd
    cmp cd, 0x00FF  ; reached bottom of user memory?
    bne RAM_FILL    ; if not, keep going

RAM_READ:
    inc cd          ; pointer to next test address
    cmp cd, 0xFFFE  ; past top?
    beq DONE        ; if so, all memory is present and OK
    dec (cd)        ; 0x02 goes to 0x01
    beq DONE        ; but if zero, RAM is faulty
    inc (0x0000)    ; anti-aliasing tripwire
    dec (cd)        ; 0x01 goes to 0x00
    beq RAM_READ    ; if zero, RAM OK: test next location

DONE:
    dec cd          ; cd points to the highest usable address
```

Whitespace is ignored, and only `0x`-prefixed hexadecimal numbers are recognised as literals.

# Changelog

* **0.5.0** — `org` and `data` directives, traps implemented, `trap`, `rti`, `call`, `ret`, `ld (RR), R`, `ld R, R`, `push`, `pop`, `inc/dec (RR)`, `inc/dec (NN)`, `bra` instructions, reset vector, stack pointer, ROM binary, forward labels
* **0.4.0** — `beq`, `bne`, `inc`, `dec`, and `cmp` instructions; zero and carry flags; backward labels, comments
* **0.3.0** — all registers, load immediate and store direct instructions
* **0.2.0** — monitor improvements, add `halt` instruction, add assembler
* **0.1.0** — first release
