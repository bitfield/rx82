    ld a, 0xFF ; loop count
LOOP:
    dec a
    bne LOOP
    halt
    