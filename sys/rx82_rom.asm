; ROM entry point
    org 0xC000

; RAM test
RAM_TEST:
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
    beq RAM_DONE        ; if so, all memory is present and OK
    dec (cd)        ; 0x02 goes to 0x01
    beq RAM_DONE        ; but if zero, RAM is faulty
    inc (0x0000)    ; anti-aliasing tripwire
    dec (cd)        ; 0x01 goes to 0x00
    beq RAM_READ    ; if zero, RAM OK: test next location

RAM_DONE:
    dec cd          ; cd points to the highest usable address
    ld (0x0040), d  ; save in RAMTOP variable in system data area
    ld (0x0041), c

; safe to write to RAM now

STACK_SET:
    ld sp, cd       ; initialise stack pointer to top of available RAM

; stack is now usable
                    
INVOKE_MONITOR:
    halt
