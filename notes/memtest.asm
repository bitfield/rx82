    ld  cd, 0xFFFF  ; start at top of memory
    ld  a, 0x02
RAM_FILL:           
    ld  (cd), a     ; write 0x02 to each location
    dec cd
    cmp cd, 0x1000  ; reached bottom of user memory?
    bne RAM_FILL    ; if not, keep going

RAM_READ:
    inc cd          ; update the pointer
    cmp cd, 0x0000  ; gone past top?
    beq DONE        ; if so, done
    ld  a, (cd)
    dec a           ; 0x02 goes to 0x01
    beq DONE        ; but if zero then RAM is faulty
    ld  (cd), a     ; store it back
    ld  a, (cd)
    dec (cd)        ; 0x01 goes to 0x00
    beq RAM_READ    ; step to the next test unless it fails

DONE:
    dec cd          ; cd points to the highest usable location
