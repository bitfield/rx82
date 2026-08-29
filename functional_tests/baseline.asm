    ld a, 0x00
    cmp a, 0x00
    bne FAIL
    ld a, 0x2E ; '.'
    bra END
FAIL:
    ld a, 0x46 ; 'F'
END:
    trap 0x20
