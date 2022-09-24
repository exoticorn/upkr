.section .text

#define FRAME_SIZE (256+32*4+4)

// x8 prob array ptr
// x9 prev was literal
// x10 out ptr
// x11 in ptr
// x12 offset
// x13 state

.global upkr_unpack
.type upkr_unpack, %function
upkr_unpack:
	mv t4, ra
	mv x17, x8
	mv t6, x9
	li x13, FRAME_SIZE
	li x9, 128
1:
	addi sp, sp, -1
	sb x9, 0(sp)
	addi x13, x13, -1
	bnez x13, 1b
	
.Lmainloop:
	li x14, 0
	jal upkr_decode_bit
	beqz x15, .Lliteral
	
	slli x14, x14, 8
	beqz x9, .Lread_offset_inc_x14
	jal upkr_decode_bit
	bnez x15, .Lread_offset

.Lfinished_offset:
	addi x14, x14, 64
	jal t3, upkr_decode_number
1:
	add x14, x10, t0
	lbu x14, (x14)
.Lstore_byte:
	sb x14, (x10)
	addi x10, x10, 1
	addi x9, x9, 1
	blt x9, x0, 1b
	j .Lmainloop
	
.Lliteral:
	jal upkr_decode_bit
	addi x14, x14, -1
	slli x14, x14, 1
	add x14, x14, x15
	srli x9, x14, 8
	beqz x9, .Lliteral
	j .Lstore_byte

.Lread_offset_inc_x14:
	addi x14, x14, 1
.Lread_offset:
	jal t3, upkr_decode_number
	addi t0, x9, 1
	bnez t0, .Lfinished_offset
.Ldone:
	addi sp, sp, FRAME_SIZE
	mv x8, x17
	mv x9, t6
	jr t4

// x14 context index
// return: x9 negtive decoded number 
upkr_decode_number:
	mv t5, x14
	li x9, 0
	li x8, -1
1:
	jal upkr_decode_bit
	beqz x15, 1f
	jal upkr_decode_bit
	beqz x15, 2f
	add x9, x9, x8
2:
	slli x8, x8, 1
	j 1b
1:
	add x9, x9, x8
	
	mv x14, t5
	jr t3

upkr_load_byte:
	lbu x15, 0(x11)
	addi x11, x11, 1
	slli x13, x13, 8
	add x13, x13, x15
// x8 prob array ptr
// x11 in ptr
// x13 state
// x14 context index
// return:
//   x14 context index + 1
//   x15 decoded bit
upkr_decode_bit:
	srli x15, x13, 12
	beqz x15, upkr_load_byte

	mv t1, x14
	mv t2, x10
	
	add x14, x14, sp
	lbu x12, 0(x14)
	
	andi x10, x13, 255
	sltu x15, x10, x12
	srli x13, x13, 8
	beqz x15, .Lelse
	
	mul x13, x13, x12
	add x13, x13, x10
	li x10, 256 + 8
	sub x10, x10, x12
	srli x10, x10, 4
	add x12, x12, x10
	j .Lendif

.Lelse:
	li x16, 256
	sub x16, x16, x12
	mul x13, x13, x16
	add x13, x13, x10
	sub x13, x13, x12
	addi x10, x12, 8
	srli x10, x10, 4
	sub x12, x12, x10

.Lendif:

	sb x12, 0(x14)
	
	addi x14, t1, 1
	mv x10, t2
	ret
