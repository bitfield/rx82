use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use rx82::{asm::Assembler, monitor::Monitor};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    /// Subcommand.
    command: Option<Command>,
    /// Path of a binary file to load.
    path: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Assemble a source file.
    Asm {
        /// Enable verbose debugging.
        #[clap(short, long)]
        debug: bool,
        /// Path of the source file.
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut mon = Monitor::default();
    match cli.command {
        Some(Command::Asm { debug, path }) => {
            let source = fs::read_to_string(&path)?;
            let asm = Assembler { debug };
            let data = asm.assemble(&source)?;
            let mut bin_path = path.clone();
            bin_path.set_extension("bin");
            fs::write(bin_path, data)?;
            Ok(())
        }
        None => {
            if let Some(path) = cli.path {
                mon.load_bin(0x0000, path)?;
            }
            mon.run()
        }
    }
}
