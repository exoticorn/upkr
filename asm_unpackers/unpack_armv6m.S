.syntax unified
.thumb

.section .text

#define ALIGNUP(n, align) (((n) + (align) - 1) & ~((align) - 1))
#define PROB_LEN (1 + 255 + 1 + 2*32 + 2*32)
#define DATA_OFF ALIGNUP(PROB_LEN, 4)

// auto upkr_unpack(uint8_t * out, uint8_t * in) -> tuple<uint8_t *, uint8_t *>
.global upkr_unpack
.type upkr_unpack, %function
// r0 .. out_ptr (returned)
// r1 .. in_ptr (returned)
// r2 .. state
// r3 .. offset
// r4 .. prev_was_match
// r5 .. subroutine arg (preserved)
// r6 .. subroutine ret
// r7 .. probs ptr
upkr_unpack:
    push { r4, r5, r6, r7, lr }
    sub sp, sp, #DATA_OFF

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
    mov r4, r6 // = 1
    bne 1f

    bl upkr_decode_bit
    beq 2f

1:
    adds r5, r5, #1
    bl upkr_decode_length
    subs r5, r5, #1
    // r4 = 1
    subs r3, r4, r6
    beq .Lend
2:

    adds r5, r5, #65
    bl upkr_decode_length
.Lcopy_loop:
    ldrb r5, [r0, r3]
    strb r5, [r0]
    adds r0, r0, #1
    subs r6, r6, #1
    bne .Lcopy_loop
    b .Lloop

.Ldata:
    movs r4, #0
    movs r5, #1

.Ldata_loop:
    bl upkr_decode_bit
    lsls r5, r5, #1
    adds r5, r5, r6
    lsrs r6, r5, #8
    beq .Ldata_loop

    strb r5, [r0]
    adds r0, r0, #1
    b .Lloop

.Lend:
    add sp, sp, #DATA_OFF
    pop { r4, r5, r6, r7, pc }

.type upkr_decode_length, %function
// r0 .. length tmp (saved)
// r1 ..
// r2 ..
// r3 ..
// r4 .. bit pos (saved)
// r5 .. content index (saved)
// r6 .. length (returned)
// r7 ..
upkr_decode_length:
    push { r0, r4, r5, lr }

    movs r4, #0
    movs r0, #0
.Lbit_loop:
    bl upkr_decode_bit
    beq 1f

    adds r5, r5, #1
    bl upkr_decode_bit
    adds r5, r5, #1
    lsls r6, r6, r4
    adds r4, r4, #1
    orrs r0, r0, r6
    b .Lbit_loop
1:
    movs r6, #1
    lsls r6, r6, r4
    orrs r6, r6, r0

    pop { r0, r4, r5, pc }

.type upkr_decode_bit, %function
// r0 .. tmp / prob (saved)
// r1 .. out_ptr (modified)
// r2 .. state (modified)
// r3 .. scratch (saved)
// r4 ..
// r5 .. content index (preserved)
// r6 .. bit (returned)
// r7 .. probs ptr (preserved)
upkr_decode_bit:
    push { r0, r3, lr }

.Lstate_loop:
    lsrs r3, r2, #12
    bne 1f
    lsls r2, r2, #8
    ldrb r6, [r1]
    adds r1, r1, #1
    orrs r2, r2, r6
    b .Lstate_loop
1:

    ldrb r0, [r7, r5]

    lsrs r3, r2, #8
    uxtb r2, r2

    subs r6, r2, r0
    lsrs r6, r6, #31
    bne 1f

    subs r2, r2, r0
    rsbs r0, r0, #0
    uxtb r0, r0
1:

    muls r3, r3, r0
    adds r2, r2, r3

    rsbs r3, r0, #0
    uxtb r3, r3
    adds r3, r3, #8
    lsrs r3, r3, #4
    adds r0, r0, r3

    cmp r6, #0
    bne 1f

    rsbs r0, r0, #0
1:

    strb r0, [r7, r5]

    cmp r6, #0
    pop { r0, r3, pc }
