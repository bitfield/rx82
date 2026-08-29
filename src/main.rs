use anyhow::Result;
use clap::{Parser, Subcommand};

use std::{fs, path::PathBuf};

use rx82::{
    asm::{Disassembler, assemble_source_file},
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
        /// Skip running boot ROM.
        #[clap(long)]
        skiprom: bool,
        /// Single-step if loading a binary file.
        #[clap(short, long)]
        step: bool,
        /// Path to a binary file to load.
        path: Option<PathBuf>,
        /// Full host-native speed.
        #[clap(short, long)]
        turbo: bool,
    },
    /// Assemble and run a program in the monitor.
    Run {
        /// Skip running boot ROM.
        #[clap(long)]
        skiprom: bool,
        /// Single-step the program.
        #[clap(short, long)]
        step: bool,
        /// Path to the source file.
        path: PathBuf,
        /// Full host-native speed.
        #[clap(short, long)]
        turbo: bool,
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
        Command::Asm { path } => {
            let data = assemble_source_file(&path)?;
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
        Command::Mon {
            path: Some(path),
            skiprom,
            step,
            turbo,
        } => {
            let program = fs::read(path)?;
            let mut mon = Monitor::default();
            mon.skiprom = skiprom;
            mon.step = step;
            mon.sys.turbo = turbo;
            mon.run_program(&program)
        }
        Command::Mon {
            path: None,
            skiprom,
            turbo,
            ..
        } => {
            let mut mon = Monitor::default();
            mon.skiprom = skiprom;
            mon.step = true;
            mon.sys.turbo = turbo;
            mon.interact()
        }
        Command::Run {
            path,
            skiprom,
            step,
            turbo,
        } => {
            let program = assemble_source_file(&path)?;
            let mut mon = Monitor::default();
            mon.skiprom = skiprom;
            mon.step = step;
            mon.sys.turbo = turbo;
            mon.run_program(&program)
        }
    }
}
