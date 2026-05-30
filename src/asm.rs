use crate::instructions::INSTRUCTIONS;

#[inline]
pub fn disassemble(slice: &[u8]) -> String {
    let mut data = slice.iter();
    let Some(opcode) = data.next() else {
        return "???".to_owned();
    };
    let Some(ins) = INSTRUCTIONS.get(opcode) else {
        return "???".to_owned();
    };
    match ins.bytes {
        1 => ins.name.to_owned(),
        2 if let Some(operand) = data.next() => format!("{}, {:#04X}", ins.name, operand),
        _ => "???".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use crate::instructions::{LDA_N, NOP};

    use super::*;

    #[test]
    fn disassemble_correctly_disassembles_instructions() {
        assert_eq!(disassemble(&[NOP]), "nop");
        assert_eq!(disassemble(&[0xFF]), "???");
        assert_eq!(disassemble(&[LDA_N, 0xFF]), "ld a, 0xFF");
    }
}
