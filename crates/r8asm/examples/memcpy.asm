; copy a block of memory
    ld cd, 0x1000 ; source address
    ld ef, 0x2000 ; destination address
    ld b, 0xFF    ; bytes to copy

LOOP:
    cmp b, 0x00
    beq DONE
    ld a, (cd)
    ld (ef), a
    inc cd
    inc ef
    dec b
    bra LOOP

DONE:
    halt
