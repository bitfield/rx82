; RAM test
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
    dec (cd)        ; 0x02 goes to 0x01
    beq DONE        ; but if zero, RAM is faulty
    inc (0x0000)    ; anti-aliasing tripwire
    dec (cd)        ; 0x01 goes to 0x00
    beq RAM_READ    ; if zero, RAM OK: test next location

DONE:
    dec cd          ; cd points to the highest usable address

STACK_SET:
    ld sp, cd       ; initialise stack pointer to top of available RAM
    halt
