For boot ROM we need:
[X] Load programs at 0x0100 instead of 0x0000
[X] CPU: Stack pointer
[X] `push R` / `pop R`
[X] System: include ROM binary from file
[X] Asm/CPU: `ld R, R`
[X] CPU: reset vector
[X] ROM: run RAM test
[X] Assembler: `org` directive
[X] Asm: pad with zeroes before `org` if necessary
[X] `trap N` / `rti`
[X] ROM: set up basic trap table / handlers
[X] Assembler: `data` directive for bytes
[X] (simulated) BIOS putchar at trap 0x20
[ ] Assembler: `--zip` flag to generate zipped binaries
[ ] System: include zipped ROM binary
[ ] `data` for strings
[ ] ROM: show startup message
[ ] ROM: print decimal routine
[ ] ROM: show bytes free message
[ ] CPU: invoke monitor from trap
[ ] Monitor: load zipped binary as ROM or user program

For `to_hex` we need:
[ ] `and R, N`
[ ] `lsr R, N`
[X] `call` / `ret`

Use case programs to write:
[ ] 16-bit multiply routine
[ ] Print status flags

Other:
[X] Monitor: memory dump should show ROM contents
[X] CPU: trap on illegal instruction
[ ] Monitor: bus tracing
[ ] Monitor: disassembly
[ ] Monitor: assembly
[ ] Monitor: modify memory
[ ] Monitor: command history/editing (`rustyline`)
[ ] Assembler: define symbols (`=`)
[ ] Assembler: format source file
[ ] Assembler: file/line error reporting
[ ] Assembler: fancy error reporting (`annotate-snippets-rs`)
[ ] Disassembler: SkoolKit-style HTML cross-linked listings
[ ] System: emulated serial device
[ ] ROM: write character to serial
