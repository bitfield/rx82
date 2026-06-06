use anyhow::Result;

use rx82::monitor::Monitor;

fn main() -> Result<()> {
    Monitor::default().run()
}
