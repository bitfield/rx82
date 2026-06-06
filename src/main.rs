use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use rx82::{asm::assemble, monitor::Monitor};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,
    path: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Assemble a source file.
    Asm {
        /// Path of the source file.
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut mon = Monitor::default();
    match cli.command {
        Some(Command::Asm { path }) => {
            let source = fs::read_to_string(&path)?;
            let data = assemble(&source)?;
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
