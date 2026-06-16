For 'memtest' we need:
[ ] cd register
[ ] 'ld cd, NN'
[ ] memory write
[ ] Store immediate indirect ('ld (cd), NN')
[ ] 16-bit compare immediate ('cmp cd, NN')
[ ] Zero flag
[ ] Relative jump on zero and non-zero ('jrz E', 'jrnz E')
[ ] Labels
[ ] Indirect inc/dec ('dec (cd)')
[ ] inc/dec cd

For 'hello' we further need:
[ ] 'ld b'
[ ] load register indirect ('ld a, (cd)')
[ ] 8- and 16-bit inc and dec
[ ] 'cmp b, N'
[ ] Unconditional relative jump
[ ] Assembler '.data' directive
[ ] 'and a, N'
[ ] store register direct ('ld NN, A')
[ ] simple UART / TTY output
