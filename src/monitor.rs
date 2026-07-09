use anyhow::Result;

use std::io::{Write as _, stdin, stdout};

use crate::{
    cpu::{Phase, Target},
    system::System,
};

#[expect(clippy::non_ascii_literal, reason = "copyright symbol")]
const BANNER: &str = "RMON v1.0 © 1977 Solid State Technologies, Inc.\n";

const USAGE: &str = "Commands:
H             = Help
M [<address>] = Memory dump
S [<address>] = Single step
Q             = Quit
Enter         = Repeat last command";

/// A user command.
#[non_exhaustive]
#[derive(Copy, Clone)]
pub enum Command {
    /// Show help information.
    Help,
    /// Dump memory.
    Memory,
    /// Quit the monitor.
    Quit,
    /// Step the system by one instruction.
    Step,
}

use Command::*;

/// The interactive CLI system monitor.
#[non_exhaustive]
#[derive(Default)]
pub struct Monitor {
    /// Last address referenced.
    pub last_addr: Option<u16>,
    /// Last command entered.
    pub last_cmd: Option<Command>,
    /// Enables single-step mode.
    pub step: bool,
    /// The running system.
    pub sys: System,
}

impl Monitor {
    /// Prompts and reads the user's next command.
    ///
    /// # Errors
    ///
    /// If reading the command fails.
    fn get_command(&mut self) -> Result<(Command, Option<u16>)> {
        let mut input = String::new();
        print!("> ");
        stdout().flush()?;
        _ = stdin().read_line(&mut input)?;
        let (cmd, arg) = match input.to_ascii_uppercase().trim() {
            "" => (self.last_cmd.unwrap_or(Step), None),
            cmd if cmd.starts_with('M') => (Memory, parse_arg(cmd)),
            cmd if cmd.starts_with('S') => (Step, parse_arg(cmd)),
            cmd if cmd.starts_with('Q') => (Quit, None),
            _ => (Help, None),
        };
        self.last_cmd = Some(cmd);
        Ok((cmd, arg))
    }

    /// Prints usage information.
    #[inline]
    pub fn help(&self) {
        println!("{USAGE}");
    }

    /// Dumps memory at `addr` (default: PC).
    #[inline]
    pub fn memory(&mut self, addr: Option<u16>) {
        let mut base = addr.unwrap_or(self.last_addr.unwrap_or(self.sys.cpu.pc));
        for _ in 0..8_u8 {
            print!("{base:04X}:");
            let mut offset = 0;
            for _ in 0..16_u8 {
                let byte = self.sys.mem.get(base.wrapping_add(offset));
                print!(" {byte:02X}");
                offset = offset.wrapping_add(1);
            }
            println!();
            base = base.wrapping_add(16);
        }
        self.last_addr = Some(base);
    }

    /// Runs the monitor interactively until the user quits.
    ///
    /// # Errors
    ///
    /// If reading the user's command input fails.
    #[inline]
    pub fn run(&mut self) -> Result<()> {
        println!("{BANNER}");
        self.step = true;
        self.last_cmd = Some(Step);
        self.sys.debug_print();
        loop {
            match self.get_command()? {
                (Help, _) => self.help(),
                (Memory, addr) => self.memory(addr),
                (Step, addr) => self.step(addr),
                (Quit, _) => break,
            }
        }
        Ok(())
    }

    /// Steps the system by one instruction.
    #[inline]
    pub fn step(&mut self, addr: Option<u16>) {
        self.sys.cpu.halt = false;
        if let Some(addr) = addr {
            self.sys.cpu.pc = addr;
        }
        loop {
            self.sys.tick();
            if self.sys.cpu.halt {
                println!("halted");
                break;
            }
            if self.sys.cpu.phase == Phase::Fetch && self.sys.cpu.target == Target::Opcode {
                break;
            }
        }
        self.last_addr = Some(self.sys.cpu.pc);
        self.sys.debug_print();
    }
}

/// Extracts the argument from a user command.
fn parse_arg(cmd: &str) -> Option<u16> {
    match cmd.split_once(' ') {
        Some((_, arg)) => u16::from_str_radix(arg, 16).ok(),
        None => None,
    }
}
