; Print a byte as hex
; 
; In: 
;   A = byte
    push a
    ld a, 0x30   ; '0'
    trap 0x10    ; print
    ld a, 0x78   ; 'x'
    trap 0x10    ; print
    pop a
    push a
    lsr a, 0x04  ; take upper nibble first
    call NIBBLE_TO_ASCII
    trap 0x10    ; print
    pop a
    and a, 0x0F  ; lower nibble
    call NIBBLE_TO_ASCII
    trap 0x10    ; print
    ret
    
NIBBLE_TO_ASCII:
    add a, 0x30  ; ASCII '0'
    cmp a, 0x3A  ; digit less than 10?
    bmi NUMERAL   
    add a, 0x11  ; ASCII 'A'
NUMERAL:
    ret
