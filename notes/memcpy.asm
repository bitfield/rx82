MEMCPY:
    ld cd, 0x1000 ; source address
    ld ef, 0x2000 ; destination address
    ld gh, 0x0100 ; bytes to copy
    
LOOP:
    cmp gh, 0     ; done?
    jrz DONE
    ld a, (cd)
    ld (ef), a
    inc cd
    inc ef
    dec gh
    jr LOOP

DONE:
    halt