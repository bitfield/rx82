    def STDOUT 1
    jmp START
HELLO:
    data "Hello, world!", 0x00
START:
    ld cd, HELLO
LOOP:
    cmp (cd), 0x00
    rtz
    out (cd), STDOUT
    inc cd
    jmp LOOP
