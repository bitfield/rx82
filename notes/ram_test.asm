RAM_TEST:
    ld cd, 0xFFFF   ; start at top of memory
RAM_FILL:           
    ld (cd), 0x02   ; write 0x02 to each location
    dec cd
    cmp cd, 0x1000  ; reached bottom of user memory?
    jnz RAM_FILL    ; if not, keep going
RAM_READ:
    inc cd          ; update the pointer
    cmp cd, 0x0000  ; gone past top?
    jrz RAM_DONE    ; if so, done
    dec (cd)        ; 0x02 goes to 0x01
    jrz RAM_DONE    ; but if zero then RAM is faulty
    dec (cd)        ; 0x01 goes to 0x00
    jrz RAM_READ    ; step to the next test unless it fails
RAM_DONE:
    dec cd          ; cd points to the highest usable location
