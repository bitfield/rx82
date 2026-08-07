use crate::instructions::InstructionKind;

#[inline]
pub fn opcodes() {
    println!(
        "| HI/LO | -0 | -1 | -2 | -3 | -4 | -5 | -6 | -7 | -8 | -9 | -A | -B | -C | -D | -E | -F |"
    );
    println!(
        "| :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: |"
    );
    for hi in 0x0..=0xF_u8 {
        print!("| `{hi:X}-` |");
        for lo in 0x0..=0xF_u8 {
            let opcode = (hi << 4_u8) | lo;
            if let Ok(ins) = InstructionKind::try_from(opcode) {
                print!(" `{ins}` |");
            } else {
                print!(" `--` |");
            }
        }
        println!();
    }
}
