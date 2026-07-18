For boot ROM we need:
[ ] Load programs at 0x0100 instead of 0x0000
[ ] `trap N`
[ ] (simulated) BIOS putchar at trap 0x10
[ ] ROM: run RAM test
[ ] ROM: show startup message
[ ] ROM: print decimal routine
[ ] ROM: show bytes free message
[ ] CPU: invoke monitor from trap
[ ] CPU: reset vector
[ ] ROM: include binary from file
[ ] Assembler: Org directive
[ ] Assembler: Dw directive 

For `to_hex` we need:
[ ] CPU: Stack pointer
[ ] `push R` / `pop R`
[ ] `and R, N`
[ ] `lsr R, N`
[ ] `call` / `ret`
   
Use case programs to write:
[ ] 16-bit multiply routine
[ ] Print status flags

Other:
[ ] Monitor: memory dump should show ROM contents
[ ] Monitor: bus tracing
[ ] Monitor: disassembly
[ ] Monitor: assembly
[ ] Monitor: modify memory
[ ] Monitor: command history/editing (`rustyline`)
[ ] Assembler: define symbols (`=`)
[ ] Assembler: format source file
[ ] Assembler: file/line error reporting
[ ] Assembler: fancy error reporting (`annotate-snippets-rs`)
[ ] CPU: trap on illegal instruction
[ ] Disassembler: SkoolKit-style HTML cross-linked listings
[ ] System: emulated serial device
[ ] ROM: write character to serial
