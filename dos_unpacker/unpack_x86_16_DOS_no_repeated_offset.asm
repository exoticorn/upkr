; Contributions from pestis, TomCat and exoticorn
;
; This is the 16-bit DOS x86 decompression stub for upkr, which is designed for
; the --no-repeated-offsets option of upkr. The decompression stub is slightly
; smaller, but the compressed data might be bigger, so you have to test if
; --no-repeated-offsets pays off in the end. This stub relocates the compressed
; data so it can be decompressed starting at the normal .COM starting address.
;
; How to use:
;   1) Pack your intro using upkr into data.bin with the --x86b command line
;      argument: (notice the --x86b, not --x86!)
;
;           $ upkr --x86b intro.com data.bin
;
;   2) Compile this .asm file using nasm (or any compatible assembler):
;
;           $ nasm unpack_x86_16_DOS_no_repeated_offsets.asm -fbin -o intropck.com
;
; The packed size of the intro+stub is limited by max_len (see below) bytes.
;
; In specific cases, the unpacker stub can be further optimized to save a byte
; or two:
;   1) You can remove CLC before RET, if you don't mind carry being set upon
;      program entry
;   2) You can also move PUSHA before PUSH SI and put POPA as the first
;      operation of the compressed code.
max_len     equ 16384
prog_start  equ (0x100+max_len+510+relocation-upkr_unpack)
probs       equ (((prog_start+max_len+510)+255)/256)*256

org 0x100


; This is will be loaded at 0x100, but relocates the code and data to prog_start
relocation:
    push    si                  ; si = 0x100 at DOS start, so save it for later ret
    pusha                       ; pusha to recall all registers before starting intro
    push    si                  ; for pop di to start writing the output
    mov     di, prog_start      ; the depacker & data are relocated from 0x100 to prog_start
    mov     ch, max_len/512
    rep     movsw
    jmp     si                  ; jump to relocated upkr_unpack


; upkr_unpack unpacks the code to 0x100 and runs it when done.
upkr_unpack:
    xchg    ax, bp              ; position in bitstream = 0
    cwd                         ; upkr_state = 0;
    xchg    cx, ax              ; cx > 0x0200
    mov     al, 128             ; for(int i = 0; i < sizeof(upkr_probs); ++i) upkr_probs[i] = 128;
    rep     stosb
    pop     di                  ; u8* write_ptr = (u8*)destination;
.mainloop:
    mov     bx, probs
    call    upkr_decode_bit
    jnc     .else               ; if(upkr_decode_bit(0)) {
    inc     bh
    call    upkr_decode_number  ;  offset = upkr_decode_number(258) - 1;
    loop    .notdone            ; if(offset == 0)
    popa
    clc
    ret
.notdone:
    mov     si, di
.sub:
    dec     si
    loop    .sub
    mov     bl, 128             ; int length = upkr_decode_number(384);
    call    upkr_decode_number
    rep     movsb               ; *write_ptr = write_ptr[-offset];
    jmp     .mainloop
.else:
    inc     bx
.byteloop:
    call    upkr_decode_bit     ; int bit = upkr_decode_bit(byte);
    adc     bl, bl              ; byte = (byte << 1) + bit;
    jnc     .byteloop
    xchg    ax, bx
    stosb
    jmp     .mainloop           ;  prev_was_match = 0;


; upkr_decode_bit decodes one bit from the rANS entropy encoded bit stream.
; parameters:
;    bx = memory address of the context probability
;    dx = decoder state
;    bp = bit position in input stream
; returns:
;    dx = new decoder state
;    bp = new bit position in input stream
;    carry = bit
; trashes ax
upkr_load_bit:
    bt      [compressed_data-relocation+prog_start], bp
    inc     bp
    adc     dx, dx
upkr_decode_bit:
    inc     dx
    dec     dx              ; or whatever other test for the top bit there is
    jns     upkr_load_bit
    movzx   ax, byte [bx]   ; u16 prob = upkr_probs[context_index]
    neg     byte [bx]
    push    ax              ; save prob, tmp = prob
    cmp     dl, al          ; int bit = (upkr_state & 255) < prob ? 1 : 0; (carry = bit)
    pushf                   ; save bit flags
    jc      .bit            ; (skip if bit)
    xchg    [bx], al        ;   tmp = 256 - tmp;
.bit:
    shr     byte [bx], 4    ; upkr_probs[context_index] = tmp + (256 - tmp + 8) >> 4;
    adc     [bx], al        ; upkr_probs[context_index] = tmp;
    mul     dh              ; upkr_state = tmp * (upkr_state >> 8) + (upkr_state & 255);
    mov     dh, 0
    add     dx, ax
    popf
    pop     ax
    jc      .bit2           ; (skip if bit)
    neg     byte [bx]       ;   tmp = 256 - tmp;
    sub     dx, ax          ;   upkr_state -= prob; note that this will also leave carry always unset, which is what we want
.bit2:
    ret                     ; flags = bit


; upkr_decode_number loads a variable length encoded number (up to 16 bits) from
; the compressed stream. Only numbers 1..65535 can be encoded. If the encoded
; number has 4 bits and is 1ABC, it is encoded using a kind of an "interleaved
; elias code": 0A0B0C1. The 1 in the end implies that no more bits are coming.
; parameters:
;   cx = must be 0
;   bx = memory address of the context probability
;   dx = decoder state
;   bp = bit position in input stream
;   carry = must be 1
; returns:
;   cx = length
;   dx = new decoder state
;   bp = new bit position in input stream
;   carry = 1
; trashes bl, ax
upkr_decode_number_loop:
    inc     bx
    call    upkr_decode_bit
upkr_decode_number:
    rcr     cx, 1
    inc     bx
    call    upkr_decode_bit
    jnc     upkr_decode_number_loop     ; 0 = there's more bits coming, 1 = no more bits
.loop2:
    rcr     cx, 1
    jnc     .loop2
    ret


compressed_data:
    incbin "data.bin"
