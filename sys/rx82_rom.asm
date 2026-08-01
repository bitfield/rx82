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
    beq RAM_DONE    ; if so, all memory is present and OK
    dec (cd)        ; 0x02 goes to 0x01
    beq RAM_DONE    ; but if zero, RAM is faulty
    inc (0x0000)    ; anti-aliasing tripwire
    dec (cd)        ; 0x01 goes to 0x00
    beq RAM_READ    ; if zero, RAM OK: test next location

RAM_DONE:
    dec cd          ; cd points to the highest usable address
    ld 0x0040, d  ; save in RAMTOP variable in system data area
    ld 0x0041, c

; safe to write to RAM now

STACK_SET:
    ld sp, cd       ; initialise stack pointer to top of available RAM

; stack is now usable

; set up trap table
TRAP_INIT:
    ; first set all vectors to the 'unhandled' handler
    ld cd, UNHANDLED_TRAP
    ld b, 0x40
    ld ef, 0x0000 ; base of trap table
STORE_DEFAULT_VECTOR:
    ld (ef), d
    inc ef
    ld (ef), c
    inc ef
    dec b
    bne STORE_DEFAULT_VECTOR

    ; set up the 'illegal instruction' handler
    ld cd, ILLEGAL_INSTRUCTION
    ld 0x0000, d  ; trap 0x00 (illegal instruction)
    ld 0x0001, c

    ; set up the 'putchar' handler
    ld cd, PUTCHAR
    ld 0x0040, d  ; vector 0x20
    ld 0x0041, c

PRINT_COPYRIGHT:
    ld cd, COPYRIGHT_MSG
    call PRINT_STRING

INVOKE_MONITOR:
    halt

; subroutines
PRINT_STRING:
    ; cd: pointer to length-prefixed string
    ld b, (cd)
NEXT_CHAR:
    inc cd
    ld a, (cd)
    trap 0x20
    dec b
    bne NEXT_CHAR
    ret

; trap handlers
ILLEGAL_INSTRUCTION:
    ld cd, ILLEGAL_INSTRUCTION_MSG
    call PRINT_STRING
    halt

UNHANDLED_TRAP:
    halt

PUTCHAR:
    rti

; data
COPYRIGHT_MSG:
    data 0x23, "(C) 1982 RX Computers Ltd.", 0x0A, "Ready.", 0x0A, 0x0A

ILLEGAL_INSTRUCTION_MSG:
    data 0x14, "illegal instruction", 0x0A

; reset vector
    org 0xFFFE
    data 0x00, 0xC0
