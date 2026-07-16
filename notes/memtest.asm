    ld  cd, 0xFFFD  ; top of possible RAM
    ld  a, 0x02

RAM_FILL:
    ld  (cd), a     ; write 0x02 to each location
    dec cd
    cmp cd, 0x00FF  ; reached bottom of user memory?
    bne RAM_FILL    ; if not, keep going

RAM_READ:
    inc cd          ; pointer to next test address
    cmp cd, 0xFFFE  ; past top?
    beq DONE        ; if so, all memory is present and OK
    ld  a, (cd)     ; get contents of address
    cmp a, 0x02     ; should be 0x02
    bne DONE        ; if not, RAM is faulty
    dec a           ; 0x02 goes to 0x01
    ld  (cd), a     ; store 0x01 back in RAM
    ld  a, (cd)     ; get contents again
    cmp a, 0x01     ; should be 0x01
    bne DONE        ; if not, RAM is faulty
    dec a           ; 0x01 goes to 0x00
    ld  (cd), a     ; store 0x00 back in RAM
    ld  a, (cd)     ; get contents again
    cmp a, 0x00     ; should be 0x00
    beq RAM_READ    ; test next location

DONE:
    dec cd          ; cd points to the highest usable address
