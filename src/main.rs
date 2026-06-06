use anyhow::Result;
use clap::Parser;

use rx82::monitor::Monitor;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Args {
    path: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut mon = Monitor::default();
    if let Some(path) = args.path {
        mon.load_bin(0x0000, path)?;
    }
    mon.run()
}
