For boot ROM we need:
[ ] Assembler: `--zip` flag to generate zipped binaries
[ ] System: include zipped ROM binary
[ ] ROM: print decimal routine
[ ] ROM: show bytes free message
[ ] CPU: invoke monitor from trap
[ ] Monitor: load zipped binary as ROM or user program

For `to_hex` we need:
[X] `and R, N`
[X] `lsr R, N`
[X] `call` / `ret`
[X] `add R, N`
[ ] `bcc D`

Use case programs to write:
[ ] 16-bit multiply routine
[ ] Print status flags

Other:
[ ] Monitor: bus tracing
[ ] Monitor: disassembly
[ ] Monitor: assembly
[ ] Monitor: modify memory
[ ] Monitor: command history/editing (`rustyline`)
[ ] System: “turbo mode”
[ ] Assembler: define symbols (`=`)
[ ] Assembler: format source file
[ ] Asm: report program size
[ ] Asm: decimal literals
[ ] Assembler: fancy error reporting (`annotate-snippets-rs`)
[ ] Disassembler: SkoolKit-style HTML cross-linked listings
[ ] System: emulated serial device
[ ] ROM: write character to serial
[ ] CPU: generate state transition diagram
