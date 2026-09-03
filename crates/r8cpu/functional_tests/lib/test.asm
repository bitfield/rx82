TEST_PASS:
    ld a, 0x2E ; '.'
    bra TEST_END
TEST_FAIL:
    ld a, 0x46 ; 'F'
TEST_END:
    trap 0x20
