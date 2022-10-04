.section .text

// x9 prev was literal
// x10 out ptr
// x11 in ptr
// x12 offset
// x13 state
// x14 context index

.global upkr_unpack
.type upkr_unpack, %function
upkr_unpack:
	mv t4, ra
	mv x17, x8
	mv t6, x9
	li x9, 256 + 128
	mv x13, x9
1:
	sub x8, sp, x13
	sb x9, 0(x8)
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
	jalr ra // jal upkr_decode_number
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
	jalr ra // jal upkr_decode_number
	addi t0, x9, 1
	bnez t0, .Lfinished_offset
.Ldone:
	mv x8, x17
	mv x9, t6
	jr t4

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

	addi x14, x14, 1
	
	sub t2, sp, x14
	lbu x12, (t2)
	
	andi x8, x13, 255
	sltu x15, x8, x12
	beqz x15, 1f
	xori x12, x12, 255
	addi x12, x12, 1
1:
	srli x8, x13, 8
	addi x8, x8, 1
	sub x8, x8, x15
	mul x8, x8, x12
	sub x13, x13, x8
	
	addi x8, x12, 8
	srli x8, x8, 4
	sub x12, x12, x8
	beqz x15, 1f
	sub x12, x0, x12
1:

	sb x12, (t2)
	
	jalr ra

// x14 context index
// return: x9 negtive decoded number 
upkr_decode_number:
	mv t3, ra
	mv t5, x14
	li x9, 0
	li t1, -1
1:
	jal upkr_decode_bit
	beqz x15, 1f
	jal upkr_decode_bit
	beqz x15, 2f
	add x9, x9, t1
2:
	add t1, t1, t1
	j 1b
1:
	add x9, x9, t1
	
	mv x14, t5
	jr t3
