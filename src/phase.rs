use core::fmt::{Display, Formatter, Result};

/// The phase of the CPU.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Phase {
    /// Decodes the opcode on the data bus.
    Decode,
    /// Executes the current instruction.
    Execute,
    /// Requests the next opcode from memory.
    #[default]
    Fetch,
    /// Waits for memory to respond.
    MemWait,
}

impl Display for Phase {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match *self {
                Phase::Decode => "DCOD",
                Phase::Execute => "EXEC",
                Phase::Fetch => "FTCH",
                Phase::MemWait => "WAIT",
            }
        )
    }
}
