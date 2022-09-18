// armv6-m upkr unpacker by yrlf
// some optimizations by exoticorn

.syntax unified
.thumb

.section .text

#define ALIGNUP(n, align) (((n) + (align) - 1) & ~((align) - 1))
#define PROB_LEN (1 + 255 + 1 + 2*32 + 2*32)
#define FRAME_SIZE ALIGNUP(PROB_LEN, 4)

// auto upkr_unpack(uint8_t * out, uint8_t * in) -> tuple<uint8_t *, uint8_t *>
.global upkr_unpack
.type upkr_unpack, %function
// r0 .. out_ptr (returned)
// r1 .. in_ptr (returned)
// r2 .. state
// r3 .. offset
// r4 .. prev_was_literal / decode_length ret
// r5 .. subroutine arg (preserved)
// r6 .. decode_bit ret
// r7 .. probs ptr
upkr_unpack:
    push { r4, r5, r6, r7, lr }
    sub sp, sp, #FRAME_SIZE

    mov r7, sp
    movs r2, #255
    adds r2, r2, #(PROB_LEN - 255)
    movs r3, #128
.Lclear:
    subs r2, r2, #1
    strb r3, [r7, r2]
    bne .Lclear

.Lloop:
    movs r5, #0
    bl upkr_decode_bit
    beq .Ldata
.Lmatch:
    // r6 = 1
    lsls r5, r6, #8
    cmp r4, #0
    beq 1f

    bl upkr_decode_bit
    beq 2f

1:
    bl upkr_decode_length
    adds r3, r4, #1
    beq .Lend
2:

    adds r5, r5, #64
    bl upkr_decode_length
.Lcopy_loop:
    ldrb r5, [r0, r3]
.Lstore:
    strb r5, [r0]
    adds r0, r0, #1
    adds r4, r4, #1
    blt .Lcopy_loop
    b .Lloop

.Ldata:
    movs r5, #1

.Ldata_loop:
    bl upkr_decode_bit
    adcs r5, r5, r5
    lsrs r4, r5, #8
    beq .Ldata_loop
    b .Lstore

.Lend:
    add sp, sp, #FRAME_SIZE
    pop { r4, r5, r6, r7, pc }

.type upkr_decode_length, %function
// r0 .. -length tmp (saved)
// r1 ..
// r2 ..
// r3 ..
// r4 .. -length (returned)
// r5 .. context index (saved)
// r6 .. (saved)
// r7 ..
upkr_decode_length:
    push { r0, r5, r6, lr }

    movs r0, #0
    subs r4, r0, #1
.Lbit_loop:
    adds r5, r5, #1
    bl upkr_decode_bit
    beq 1f

    adds r5, r5, #1
    bl upkr_decode_bit
    beq 2f
    adds r0, r0, r4
2:
    lsls r4, r4, #1
    b .Lbit_loop
1:
    adds r4, r4, r0

    pop { r0, r5, r6, pc }

.type upkr_decode_bit, %function
// r0 .. tmp / prob (saved)
// r1 .. in_ptr (modified)
// r2 .. state (modified)
// r3 .. scratch (saved)
// r4 ..
// r5 .. context index (preserved)
// r6 .. bit (returned)
// r7 .. probs ptr (preserved)
upkr_fill_state:
    lsls r2, r2, #8
    ldrb r6, [r1]
    adds r1, r1, #1
    orrs r2, r2, r6

upkr_decode_bit:
    lsrs r6, r2, #12
    beq upkr_fill_state

    push { r0, r1, r3, lr }

    ldrb r0, [r7, r5]

    lsrs r3, r2, #8
    uxtb r1, r2

    subs r6, r1, r0
    blt 1f

    subs r1, r2, r0
    rsbs r0, r0, #0
1:

    muls r3, r3, r0
    adds r2, r1, r3

    rsbs r3, r0, #0
    uxtb r3, r3
    lsrs r3, r3, #4
    adcs r0, r0, r3

    cmp r6, #0
    blt 1f

    rsbs r0, r0, #0
1:

    strb r0, [r7, r5]

    lsrs r6, r6, #31
    pop { r0, r1, r3, pc }
