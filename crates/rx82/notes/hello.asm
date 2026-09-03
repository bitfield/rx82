    ld b, 0x0D // length of string
    ld cd, HELLO

LOOP:
    cmp b, 0x00
    jrz END

    ld a, (cd)
    // spin on UART status register
    // write A to UART TX register
    
    inc cd
    dec b
    jr LOOP

HELLO: .data "Hello, world!"

END:
    halt
