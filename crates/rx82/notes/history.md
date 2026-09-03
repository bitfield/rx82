History:

For `countdown` we needed:
[X] decrement register (`dec a`)
[X] Zero flag
[X] Relative jump on non-zero (`bne D`)
[X] Backward labels

For `memcpy` we needed:
[X] cd register
[X] load register direct 'ld cd, NN'
[X] inc/dec reg pair (`inc cd`)
[X] Relative jump on zero (`beq D`)
[X] 8-bit compare immediate (`cmp b, N`)
[X] Store register indirect (`ld (cd), a`)
[X] load register indirect (`ld a, (cd)`)
[X] Unconditional relative jump (`bra D`)
[X] Forward labels

For `memtest` we needed:
[X] memory write
[X] 16-bit compare immediate (`cmp cd, NN`)
[X] indirect inc/dec (`dec (cd)`)

For boot ROM we needed:
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
[X] `data` for strings
[X] ROM: show startup message

For `to_hex` we needed:
[X] `and R, N`
[X] `lsr R, N`
[X] `call` / `ret`
[X] `add R, N`
[X] `bcc D`

Other:
[X] Monitor: memory dump should show ROM contents
[X] CPU: trap on illegal instruction
[X] Assembler: file/line error reporting
[X] CPU: generate opcode table
