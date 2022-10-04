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
    movs r3, #128
.Lclear:
    subs r2, r2, #1
    strb r3, [sp, -r2]
    bne .Lclear

.Lloop:
    movs r5, #0
    bl upkr_decode_bit
    bcc .Ldata
.Lmatch:
	mov r5, #256
    cmp r4, #0
    beq 1f

    bl upkr_decode_bit
    bcc 2f

1:
    bl upkr_decode_length
    subs r3, r4, #1
    popeq { r3-r11, pc }
2:

	mov r5, #256+64
    bl upkr_decode_length
.Lcopy_loop:
    ldrb r5, [r0, -r3]
.Lstore:
    strb r5, [r0], #1
    subs r4, r4, #1
    bgt .Lcopy_loop
    b .Lloop

.Ldata:
    movs r5, #1

.Ldata_loop:
    bl upkr_decode_bit
    adcs r5, r5, r5
	rsbs r4, r5, #256
    bgt .Ldata_loop
	b .Lstore

.type upkr_decode_length, %function
upkr_decode_length:
	mov r12, lr

    movs r6, #0
	mov r4, #1
.Lbit_loop:
    adds r5, r5, #1
    bl upkr_decode_bit
	addcc r4, r4, r6
	movcc pc, r12

    adds r5, r5, #1
    bl upkr_decode_bit
    addcs r6, r6, r4
    lsls r4, r4, #1
    b .Lbit_loop

.type upkr_decode_bit, %function
upkr_fill_state:
    ldrb r8, [r1], #1
	orr r2, r8, r2, lsl #8

upkr_decode_bit:
	cmp r2, #4096
	blt upkr_fill_state

    ldrb r8, [sp, -r5]
	and r9, r2, #255
	add r9, r9, #1
	cmp r8, r9
	rsbcs r8, r8, #256
	mov r9, r2, lsr#8
	addcc r9, r9, #1
	mul r9, r8, r9
	sub r2, r2, r9
	add r9, r8, #8
	sub r8, r8, r9, lsr#4
	rsbcs r8, r8, #256
	strb r8, [sp, -r5]
	mov pc, r14

