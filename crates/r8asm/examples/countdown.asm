; countdown timer
    ld a, 0x0B ; = about 2 seconds at 4MHz
LOOP:
    ld cd, 0xFFFF ; inner loop
INNER:
    dec cd
    bne INNER
    dec a
    bne LOOP
    halt
