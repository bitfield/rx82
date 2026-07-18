; countdown timer
    ld a, 0x06 ; = about 1 second at 4MHz
LOOP:
    ld cd, 0xFFFF ; inner loop
INNER:
    dec cd
    bne INNER
    dec a
    bne LOOP
    halt
