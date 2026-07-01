    ld a, 0x06 ; about 1 second
LOOP:
    ld cd, 0xFFFF ; inner loop
INNER:
    dec cd
    bne INNER
    dec a
    bne LOOP
    halt
