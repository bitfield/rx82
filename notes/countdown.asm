    ld a, 0xFF ; loop count
LOOP:
    dec a
    jrnz LOOP
    halt
    