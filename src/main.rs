use anyhow::Result;
use clap::{Parser, Subcommand};

use std::{fs, path::PathBuf};

use rx82::{
    asm::{Assembler, Disassembler},
    doc::opcodes,
    monitor::Monitor,
};

use crate::DocCommand::Opcodes;

/// An emulator for the RX82 fantasy retro computer system.
#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Assemble a source file.
    Asm {
        /// Enable verbose debugging.
        #[clap(short, long)]
        debug: bool,
        /// Path to the source file.
        path: PathBuf,
    },
    /// Disassemble a binary file.
    Dis {
        /// Path to the binary file.
        path: PathBuf,
    },
    /// Generate documentation.
    Doc {
        #[clap(subcommand)]
        doc_cmd: DocCommand,
    },
    /// Start the interactive monitor.
    Mon {
        /// Path to a binary file to load.
        path: Option<PathBuf>,
    },
    /// Assemble and load a source file into the monitor.
    Run {
        /// Path to the source file.
        path: PathBuf,
    },
}

#[derive(Clone, Debug, Subcommand)]
enum DocCommand {
    /// Generate opcode table.
    Opcodes,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Asm { debug, path } => {
            let source = fs::read_to_string(&path)?;
            let mut asm = Assembler::from(source.as_str());
            asm.debug = debug;
            asm.path = path.display().to_string();
            let data = asm.assemble()?;
            let mut bin_path = path.clone();
            bin_path.set_extension("bin");
            fs::write(bin_path, data)?;
            Ok(())
        }
        Command::Dis { path } => {
            let code = fs::read(&path)?;
            for source in Disassembler::from(code.as_slice()) {
                println!("    {source}");
            }
            Ok(())
        }
        Command::Doc { doc_cmd } => {
            match doc_cmd {
                Opcodes => opcodes(),
            }
            Ok(())
        }
        Command::Mon { path: Some(path) } => {
            let mut mon = Monitor::default();
            let data = fs::read(path)?;
            mon.sys.mem.load(0x0100, &data)?;
            mon.sys.cpu.pc = 0x0100;
            mon.interact()
        }
        Command::Mon { path: None, .. } => Monitor::default().interact(),
        Command::Run { path } => {
            let source = fs::read_to_string(&path)?;
            let data = Assembler::from(source.as_str()).assemble()?;
            let mut mon = Monitor::default();
            mon.sys.mem.load(0x0100, &data)?;
            mon.sys.cpu.pc = 0x0100;
            mon.interact()
        }
    }
}
