For 'hello' we need:
* The b and cd registers
* 'ld b' and 'ld cd'
* load register indirect ('ld a, (cd)')
* 8- and 16-bit inc and dec
* 8-bit cmp
* Absolute jump
* Zero flag
* Relative jump on zero
* Assembler 'data' directive
* Labels

For 'memtest' we need:
* cd register
* Store indirect ('ld (cd), 0x02')
* 16-bit cmp
* Zero flag
* Relative jump on zero and non-zero
* Indirect inc/dec ('dec (cd)')
* dec cd
