For `countdown` we need:
[ ] decrement register (`dec a`)
[ ] Zero flag
[ ] Relative jump on non-zero (`jrnz E`)
[ ] Labels

For `memcpy` we need:
[X] cd register
[X] load register direct 'ld cd, NN'
[ ] Store indirect (`ld (cd), N`)
[ ] load register indirect (`ld a, (cd)`)
[ ] 8-bit compare immediate (`cmp b, N`)
[ ] inc/dec reg pair (`inc cd`)
[ ] Unconditional relative jump (`jr`)
[ ] Relative jump on zero (`jrz E`)

For `memtest` we need:
[X] memory write
[ ] 16-bit compare immediate (`cmp cd, NN`)
[ ] Indirect inc/dec (`dec (cd)`)

For `hello` we further need:
[X] `ld b`
[ ] Unconditional relative jump
[ ] Assembler `.data` directive
[ ] `and a, N`
[ ] simple UART / TTY output
