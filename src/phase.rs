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
    FetchOpcode,
    /// Requests an operand from memory.
    FetchOperand,
    /// Waits for memory to respond.
    MemWait,
    /// Reads an operand from the data bus.
    ReadOperand,
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
                Phase::FetchOpcode => "FTCH",
                Phase::FetchOperand => "FOPR",
                Phase::MemWait => "WAIT",
                Phase::ReadOperand => "ROPR",
            }
        )
    }
}
