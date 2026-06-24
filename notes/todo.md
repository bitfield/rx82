For `memcpy` we need:
[X] cd register
[X] load register direct 'ld cd, NN'
[X] store register direct (`ld NN, A`)
[ ] Zero flag
[ ] inc/dec reg pair (`inc cd`)
[ ] Unconditional relative jump (`jr`)
[ ] Relative jump on zero (`jrz E`)
[ ] Labels

For `memtest` we need:
[X] memory write
[ ] 16-bit compare immediate (`cmp cd, NN`)
[ ] Relative jump on non-zero (`jrnz E`)
[ ] Indirect inc/dec (`dec (cd)`)

For `hello` we further need:
[X] `ld b`
[ ] load register indirect (`ld a, (cd)`)
[ ] 8- and 16-bit inc and dec
[ ] `cmp b, N`
[ ] Unconditional relative jump
[ ] Assembler `.data` directive
[ ] `and a, N`
[ ] simple UART / TTY output
