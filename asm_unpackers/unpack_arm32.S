.arm

.section .text

.global upkr_unpack
.type upkr_unpack, %function
// r0 .. out_ptr (returned)
// r1 .. in_ptr (returned)
// r2 .. state
// r3 .. offset
// r4 .. prev_was_literal / decode_length ret
// r5 .. context index
// r6 .. decode_length temp
// r7 .. probs ptr
// r8-r11 .. decode_bit temp
// r12 .. decode_length return address
upkr_unpack:
    push { r3-r11, lr }

    mov r2, #384
    mov r3, #128
.Lclear:
    subs r2, r2, #1
    strb r3, [sp, -r2]
    bne .Lclear

.Lloop:
    mov r5, #0
    bl upkr_decode_bit
    bcc .Ldata
.Lmatch:
    mov r5, #256
    rsbs r6, r4, #0
    blcc upkr_decode_bit
    bcc .Lskip_offset

    bl upkr_decode_length
    adds r3, r4, #1
    popeq { r3-r11, pc }
.Lskip_offset:

    mov r5, #256+64
    bl upkr_decode_length
.Lcopy_loop:
    ldrb r5, [r0, r3]
.Lstore:
    strb r5, [r0], #1
    adds r4, r4, #1
    blt .Lcopy_loop
    b .Lloop

.Ldata:
    mov r5, #1

.Ldata_loop:
    bl upkr_decode_bit
    adc r5, r5, r5
    movs r4, r5, lsr #8
    beq .Ldata_loop
    b .Lstore

.type upkr_decode_length, %function
upkr_decode_length:
    mov r12, lr

    mov r4, #0
    mvn r6, #0
.Lbit_loop:
    bl upkr_decode_bit_inc
    addcc r4, r4, r6
    movcc pc, r12

    bl upkr_decode_bit_inc
    addcs r4, r4, r6
    mov r6, r6, lsl #1
    b .Lbit_loop

.type upkr_decode_bit, %function
upkr_decode_bit_inc:
    add r5, r5, #1
upkr_decode_bit:
    cmp r2, #4096
    ldrltb r8, [r1], #1
    orrlt r2, r8, r2, lsl#8
    blt upkr_decode_bit

    ldrb r8, [sp, -r5]
    and r9, r2, #255
    add r9, r9, #1
    cmp r8, r9
    rsbcs r8, r8, #256
    mvn r9, r2, lsr#8
    addcs r9, r9, #1
    mla r2, r8, r9, r2
    add r9, r8, #8
    sub r8, r8, r9, lsr#4
    rsbcs r8, r8, #256
    strb r8, [sp, -r5]
    mov pc, r14

