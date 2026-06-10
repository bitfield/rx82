use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use rx82::{asm::Assembler, monitor::Monitor};

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
    #[command(flatten)]
    opts: SharedOptions,
}

#[derive(Args, Debug)]
struct SharedOptions {
    /// Enable verbose debugging.
    #[clap(short, long)]
    debug: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Assemble a source file.
    Asm {
        /// Path to the source file.
        path: PathBuf,
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Asm { path } => {
            let data = assemble(&path, cli.opts.debug)?;
            let mut bin_path = path.clone();
            bin_path.set_extension("bin");
            fs::write(bin_path, data)?;
            Ok(())
        }
        Command::Mon { path: Some(path) } => {
            let mut mon = Monitor::default();
            let data = fs::read(path)?;
            mon.load(0x0000, &data)?;
            mon.debug = cli.opts.debug;
            mon.run()
        }
        Command::Mon { path: None } => {
            let mut mon = Monitor::default();
            mon.debug = true;
            mon.run()
        }
        Command::Run { path } => {
            let data = assemble(&path, cli.opts.debug)?;
            let mut mon = Monitor::default();
            mon.load(0x0000, &data)?;
            mon.debug = cli.opts.debug;
            mon.run()
        }
    }
}

fn assemble(path: impl AsRef<Path>, debug: bool) -> Result<Vec<u8>> {
    let source = fs::read_to_string(&path)?;
    let mut asm = Assembler::new(&source);
    asm.debug = debug;
    asm.assemble()
}
