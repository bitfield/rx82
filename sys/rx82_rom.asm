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

PRINT_BYTES_FREE:
    ld ab, sp     ; highest usable RAM address
    clc
    add b, 0x01   ; bytes free is one more than that
    add a, 0x00   ; propagate carry
    sec
    sub a, 0x01   ; minus 256 bytes for trap table / data area
    call PRINT_HEX
    ld cd, READY_MSG
    call PRINT_STRING

INVOKE_MONITOR:
    halt

; subroutines

; print length-prefixed string at (cd)
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

; print word in ab as hex
PRINT_HEX:
    push a
    ld a, 0x30   ; '0'
    trap 0x20    ; print
    ld a, 0x78   ; 'x'
    trap 0x20    ; print
    pop a
    push a
    ; high byte
    lsr a, 0x04  ; take upper nibble first
    call NIBBLE_TO_ASCII
    trap 0x20    ; print
    pop a
    and a, 0x0F  ; lower nibble
    call NIBBLE_TO_ASCII
    trap 0x20    ; print
    ; low byte
    ld a, b
    push a
    lsr a, 0x04  ; take upper nibble first
    call NIBBLE_TO_ASCII
    trap 0x20    ; print
    pop a
    and a, 0x0F  ; lower nibble
    call NIBBLE_TO_ASCII
    trap 0x20    ; print
    ret
NIBBLE_TO_ASCII:
    clc
    add a, 0x30  ; ASCII '0'
    cmp a, 0x3A  ; digit less than 10?
    bcc NUMERAL
    add a, 0x06  ; ASCII 'A'
NUMERAL:
    ret

; trap handlers

; illegal instruction
ILLEGAL_INSTRUCTION:
    ld cd, ILLEGAL_INSTRUCTION_MSG
    call PRINT_STRING
    halt

; default unhandled trap handler
UNHANDLED_TRAP:
    halt

; print char in a (faked by emulator for now)
PUTCHAR:
    rti

; data
COPYRIGHT_MSG:
    data 0x1C, "(C) 1982 RX Computers Ltd.", 0x0A, 0x0A

READY_MSG:
    data 0x15, " bytes free. Ready.", 0x0A, 0x0A

ILLEGAL_INSTRUCTION_MSG:
    data 0x14, "illegal instruction", 0x0A

; reset vector
    org 0xFFFE
    data 0x00, 0xC0
