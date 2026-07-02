For `countdown` we need:
[X] decrement register (`dec a`)
[X] Zero flag
[X] Relative jump on non-zero (`bne D`)
[ ] Forward labels

For `memcpy` we need:
[X] cd register
[X] load register direct 'ld cd, NN'
[ ] Store indirect (`ld (cd), N`)
[ ] load register indirect (`ld a, (cd)`)
[ ] 8-bit compare immediate (`cmp b, N`)
[X] inc/dec reg pair (`inc cd`)
[ ] Unconditional relative jump (`bra D`)
[X] Relative jump on zero (`beq D`)
[ ] Backward labels

For `memtest` we need:
[X] memory write
[ ] 16-bit compare immediate (`cmp cd, NN`)
[ ] Maybe: indirect inc/dec (`dec (cd)`)

For `hello` we further need:
[X] `ld b`
[ ] Assembler `.data` directive
[ ] `and a, N`
[ ] simple UART / TTY output
