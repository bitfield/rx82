    ld a, 0x00
    cmp a, 0x00
    bne TEST_FAIL
    
    include lib/test.asm
