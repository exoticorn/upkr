;; https://github.com/exoticorn/upkr/blob/z80/c_unpacker/unpack.c - original C implementation
;; C source in comments ahead of asm - the C macros are removed to keep only bitstream variant
;;
;; initial version by Peter "Ped" Helcmanovsky (C) 2022, licensed same as upkr project ("unlicensed")
;; to assemble use z00m's sjasmplus: https://github.com/z00m128/sjasmplus
;;
;; you can define UPKR_PROBS_ORIGIN to specific 256 byte aligned address for probs array (386 bytes),
;; otherwise it will be positioned after the unpacker code (256 aligned)
;;
;; public API:
;;
;;     upkr.unpack
;;         IN: IX = packed data, DE' (shadow DE) = destination
;;         OUT: IX = after packed data
;;         modifies: all registers except IY, requires 10 bytes of stack space
;;

;     DEFINE BACKWARDS_UNPACK         ; uncomment to build backwards depacker
            ; initial IX points at last byte of compressed data
            ; initial DE' points at last byte of unpacked data

;     DEFINE UPKR_UNPACK_SPEED        ; uncomment to get larger but faster unpack routine

    OPT push reset --syntax=abf
    MODULE upkr

NUMBER_BITS     EQU     16+15       ; context-bits per offset/length (16+15 for 16bit offsets/pointers)
    ; numbers (offsets/lengths) are encoded like: 1a1b1c1d1e0 = 0000'0000'001e'dbca

/*
u8* upkr_data_ptr;
u8 upkr_probs[1 + 255 + 1 + 2*32 + 2*32];
u16 upkr_state;
u8 upkr_current_byte;
int upkr_bits_left;

int upkr_unpack(void* destination, void* compressed_data) {
    upkr_data_ptr = (u8*)compressed_data;
    upkr_state = 0;
    upkr_bits_left = 0;
    for(int i = 0; i < sizeof(upkr_probs); ++i)
        upkr_probs[i] = 128;

    u8* write_ptr = (u8*)destination;

    int prev_was_match = 0;
    int offset = 0;
    for(;;) {
        if(upkr_decode_bit(0)) {
            if(prev_was_match || upkr_decode_bit(256)) {
                offset = upkr_decode_length(257) - 1;
                if(offset == 0) {
                    break;
                }
            }
            int length = upkr_decode_length(257 + 64);
            while(length--) {
                *write_ptr = write_ptr[-offset];
                ++write_ptr;
            }
            prev_was_match = 1;
        } else {
            int byte = 1;
            while(byte < 256) {
                int bit = upkr_decode_bit(byte);
                byte = (byte << 1) + bit;
            }
            *write_ptr++ = byte;
            prev_was_match = 0;
        }
    }

    return write_ptr - (u8*)destination;
}
*/
; IN: IX = compressed_data, DE' = destination
unpack:
  ; ** reset probs to 0x80, also reset HL (state) to zero, and set BC to probs+context 0
    ld      hl,probs.c>>1
    ld      bc,probs.e
    ld      a,$80
.reset_probs:
    dec     bc
    ld      (bc),a              ; will overwrite one extra byte after the array because of odd length
    dec     bc
    ld      (bc),a
    dec     l
    jr      nz,.reset_probs
    exa
    ; BC = probs (context_index 0), state HL = 0, A' = 0x80 (no source bits left in upkr_current_byte)

  ; ** main loop to decompress data
    ; D = prev_was_match = uninitialised, literal is expected first => will reset D to "false"
    ; values for false/true of prev_was_match are: false = high(probs), true = 1 + high(probs)
.decompress_data:
    ld      c,0
    call    decode_bit          ; if(upkr_decode_bit(0))
    jr      c,.copy_chunk

  ; * extract byte from compressed data (literal)
    inc     c                   ; C = byte = 1 (and also context_index)
.decode_byte:
    call    decode_bit          ; bit = upkr_decode_bit(byte);
    rl      c                   ; byte = (byte << 1) + bit;
    jr      nc,.decode_byte     ; while(byte < 256)
    ld      a,c
    exx
    ld      (de),a              ; *write_ptr++ = byte;
    IFNDEF BACKWARDS_UNPACK : inc de : ELSE : dec de : ENDIF
    exx
    ld      d,b                 ; prev_was_match = false
    jr      .decompress_data

  ; * copy chunk of already decompressed data (match)
.copy_chunk:
    ld      a,b
    inc     b                   ; context_index = 256
        ;             if(prev_was_match || upkr_decode_bit(256)) {
        ;                 offset = upkr_decode_length(257) - 1;
        ;                 if (0 == offset) break;
        ;             }
    cp      d                   ; CF = prev_was_match
    call    nc,decode_bit       ; if not prev_was_match, then upkr_decode_bit(256)
    jr      nc,.keep_offset     ; if neither, keep old offset
    inc     c                   ; context_index to first "number" set for offsets decoding (257)
    call    decode_number
    dec     de                  ; offset = upkr_decode_length(257) - 1;
    ld      a,d
    or      e
    ret     z                   ; if(offset == 0) break
    ld      (.offset),de
.keep_offset:
        ;             int length = upkr_decode_length(257 + 64);
        ;             while(length--) {
        ;                 *write_ptr = write_ptr[-offset];
        ;                 ++write_ptr;
        ;             }
        ;             prev_was_match = 1;
    ld      c,low(257 + NUMBER_BITS)    ; context_index to second "number" set for lengths decoding
    call    decode_number       ; length = upkr_decode_length(257 + 64);
    push    de
    exx
    ld      h,d                 ; DE = write_ptr
    ld      l,e
.offset+*:  ld      bc,0
    IFNDEF BACKWARDS_UNPACK
    sbc     hl,bc               ; CF=0 from decode_number ; HL = write_ptr - offset
    ELSE
    add     hl,bc               ; HL = write_ptr + offset
    ENDIF
    pop     bc                  ; BC = length
    IFNDEF BACKWARDS_UNPACK : ldir : ELSE : lddr : ENDIF
    exx
    ld      d,b                 ; prev_was_match = true
    djnz    .decompress_data    ; adjust context_index back to 0..255 range, go to main loop

/*
int upkr_decode_bit(int context_index) {
    while(upkr_state < 32768) {
        if(upkr_bits_left == 0) {
            upkr_current_byte = *upkr_data_ptr++;
            upkr_bits_left = 8;
        }
        upkr_state = (upkr_state << 1) + (upkr_current_byte >> 7);
        upkr_current_byte <<= 1;
        --upkr_bits_left;
    }

    int prob = upkr_probs[context_index];
    int bit = (upkr_state & 255) >= prob ? 1 : 0;

    int prob_offset = 16;
    int state_offset = 0;
    int state_scale = prob;
    if(bit) {
        state_offset = -prob;
        state_scale = 256 - prob;
        prob_offset = 0;
    }
    upkr_state = state_offset + state_scale * (upkr_state >> 8) + (upkr_state & 255);
    upkr_probs[context_index] = prob_offset + prob - ((prob + 8) >> 4);

    return bit;
}
*/
decode_bit:
  ; HL = upkr_state
  ; IX = upkr_data_ptr
  ; BC = probs+context_index
  ; A' = upkr_current_byte (!!! init to 0x80 at start, not 0x00)
  ; preserves DE
  ; ** while (state < 32768) - initial check
    push    de
    bit     7,h
    jr      nz,.state_b15_set
    exa
  ; ** while body
.state_b15_zero:
  ; HL = upkr_state
  ; IX = upkr_data_ptr
  ; A = upkr_current_byte (init to 0x80 at start, not 0x00)
    add     a,a                     ; upkr_current_byte <<= 1; // and testing if(upkr_bits_left == 0)
    jr      nz,.has_bit             ; CF=data, ZF=0 -> some bits + stop bit still available
  ; CF=1 (by stop bit)
    ld      a,(ix)
    IFNDEF BACKWARDS_UNPACK : inc ix : ELSE : dec ix : ENDIF    ; upkr_current_byte = *upkr_data_ptr++;
    adc     a,a                     ; CF=data, b0=1 as new stop bit
.has_bit:
    adc     hl,hl                   ; upkr_state = (upkr_state << 1) + (upkr_current_byte >> 7);
    jp      p,.state_b15_zero       ; while (state < 32768)
    exa
  ; ** set "bit"
.state_b15_set:
    ld      a,(bc)                  ; A = upkr_probs[context_index]
    dec     a                       ; prob is in ~7..249 range, never zero, safe to -1
    cp      l                       ; CF = bit = prob-1 < (upkr_state & 255) <=> prob <= (upkr_state & 255)
    inc     a
  ; ** adjust state
    push    bc
    ld      c,l                     ; C = (upkr_state & 255); (preserving the value)
    push    af
    jr      nc,.bit_is_0
    neg                             ; A = -prob == (256-prob), CF=1 preserved
.bit_is_0:
    ld      d,0
    ld      e,a                     ; DE = state_scale ; prob || (256-prob)
    ld      l,d                     ; H:L = (upkr_state>>8) : 0

  IFNDEF UPKR_UNPACK_SPEED

    ;; looped MUL for minimum unpack size
    ld      b,8                     ; counter
.mulLoop:
    add     hl,hl
    jr      nc,.mul0
    add     hl,de
.mul0:
    djnz    .mulLoop                ; until HL = state_scale * (upkr_state>>8), also BC becomes (upkr_state & 255)

  ELSE

    ;;; unrolled MUL for better performance, +25 bytes unpack size
    ld      b,d
    DUP     8
        add     hl,hl
        jr      nc,0_f
        add     hl,de
0:
    EDUP

  ENDIF

    add     hl,bc                   ; HL = state_scale * (upkr_state >> 8) + (upkr_state & 255)
    pop     af                      ; restore prob and CF=bit
    jr      nc,.bit_is_0_2
    dec     d                       ; DE = -prob (also D = bit ? $FF : $00)
    add     hl,de                   ; HL += -prob
    ; ^ this always preserves CF=1, because (state>>8) >= 128, state_scale: 7..250, prob: 7..250,
    ; so 7*128 > 250 and thus edge case `ADD hl=(7*128+0),de=(-250)` => CF=1
.bit_is_0_2:
 ; *** adjust probs[context_index]
    ld      e,a                     ; preserve prob
    rra                             ; + (bit<<4) ; part of -prob_offset, needs another -16
    and     $FC                     ; clear/keep correct bits to get desired (prob>>4) + extras, CF=0
    rra
    rra
    rra                             ; A = (bit<<4) + (prob>>4), CF=(prob & 8)
    adc     a,-16                   ; A = (bit<<4) - 16 + ((prob + 8)>>4) ; -prob_offset = (bit<<4) - 16
    sub     e                       ; A = (bit<<4) - 16 + ((prob + 8)>>4) - prob ; = ((prob + 8)>>4) - prob_offset - prob
    neg                             ; A = prob_offset + prob - ((prob + 8)>>4)
    pop     bc
    ld      (bc),a                  ; probs[context_index] = prob_offset + prob - ((prob + 8) >> 4);
    add     a,d                     ; restore CF = bit (D = bit ? $FF : $00 && A > 0)
    pop     de
    ret

/*
int upkr_decode_length(int context_index) {
    int length = 0;
    int bit_pos = 0;
    while(upkr_decode_bit(context_index)) {
        length |= upkr_decode_bit(context_index + 1) << bit_pos++;
        context_index += 2;
    }
    return length | (1 << bit_pos);
}
*/
decode_number:
  ; HL = upkr_state
  ; IX = upkr_data_ptr
  ; BC = probs+context_index
  ; A' = upkr_current_byte (!!! init to 0x80 at start, not 0x00)
  ; return length in DE, CF=0
    ld      de,$7FFF            ; length = 0 with positional-stop-bit
    jr      .loop_entry
.loop:
    inc     c                   ; context_index + 1
    call    decode_bit
    rr      d
    rr      e                   ; DE = length = (length >> 1) | (bit << 15);
    inc     c                   ; context_index += 2
.loop_entry:
    call    decode_bit
    jr      c,.loop
.fix_bit_pos:
    ccf                         ; NC will become this final `| (1 << bit_pos)` bit
    rr      d
    rr      e
    jr      c,.fix_bit_pos      ; until stop bit is reached (all bits did land to correct position)
    ret                         ; return with CF=0 (important for unpack routine)

    DISPLAY "upkr.unpack total size: ",/D,$-unpack

    ; reserve space for probs array without emitting any machine code (using only EQU)

    IFDEF UPKR_PROBS_ORIGIN     ; if specific address is defined by user, move probs array there
probs:      EQU ((UPKR_PROBS_ORIGIN) + 255) & -$100     ; probs array aligned to 256
    ELSE
probs:      EQU ($ + 255) & -$100                       ; probs array aligned to 256
    ENDIF
.real_c:    EQU 1 + 255 + 1 + 2*NUMBER_BITS             ; real size of probs array
.c:         EQU (.real_c + 1) & -2                      ; padding to even size (required by init code)
.e:         EQU probs + .c

    DISPLAY "upkr.unpack probs array placed at: ",/A,probs,",\tsize: ",/A,probs.c

    ENDMODULE
    OPT pop
