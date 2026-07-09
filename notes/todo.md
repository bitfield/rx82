For `countdown` we need:
[X] decrement register (`dec a`)
[X] Zero flag
[X] Relative jump on non-zero (`bne D`)
[X] Backward labels

For `memcpy` we need:
[X] cd register
[X] load register direct 'ld cd, NN'
[X] inc/dec reg pair (`inc cd`)
[X] Relative jump on zero (`beq D`)
[X] 8-bit compare immediate (`cmp b, N`)
[ ] Store register indirect (`ld (cd), a`)
[X] load register indirect (`ld a, (cd)`)
[ ] Unconditional relative jump (`bra D`)
[ ] Forward labels

For `to_hex` we need:
[ ] `trap N`
[ ] (simulated) BIOS putchar at trap 0x10
[ ] `push R` / `pop R`
[ ] `and R, N`
[ ] `lsr R, N`
[ ] `call` / `ret`
[ ] `bmi` / `bpl`
   
For `memtest` we need:
[X] memory write
[ ] 16-bit compare immediate (`cmp cd, NN`)
[ ] Maybe: indirect inc/dec (`dec (cd)`)

For `hello` we further need:
[X] `ld b`
[ ] Assembler `.data` directive
