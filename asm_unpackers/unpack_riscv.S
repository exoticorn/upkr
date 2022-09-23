.section .text

#define FRAME_SIZE (256+64*4+4)

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
	
	li x14, 256
	beqz x9, .Lread_offset
	jal upkr_decode_bit
	beqz x15, .Lskip_offset
.Lread_offset:
	jal t3, upkr_decode_number
	addi x12, x9, -1
	beqz x12, .Ldone

.Lskip_offset:
	li x14, 256+64
	jal t3, upkr_decode_number
1:
	sub x15, x10, x12
	lbu x15, (x15)
	sb x15, (x10)
	addi x10, x10, 1
	addi x9, x9, -1
	bnez x9, 1b
	j .Lmainloop
	
.Lliteral:
	li x14, 1
1:
	jal upkr_decode_bit
	slli x14, x14, 1
	add x14, x14, x15
	srli x9, x14, 8
	beqz x9, 1b
	sb x14, 0(x10)
	addi x10, x10, 1
	j .Lmainloop

.Ldone:
	addi sp, sp, FRAME_SIZE
	mv x8, x17
	mv x9, t6
	jr t4

// x14 context index
// return: x9 decoded number 
upkr_decode_number:
	mv t5, x14
	li x9, 0
	li x8, 1
1:
	addi x14, x14, 1
	jal upkr_decode_bit
	beqz x15, 1f
	addi x14, x14, 1
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
// return: x15 decoded bit
upkr_decode_bit:
	srli x15, x13, 12
	beqz x15, upkr_load_byte

	mv t0, x9
	mv t1, x14
	mv t2, x10
	
	add x14, x14, sp
	lbu x9, 0(x14)
	
	andi x10, x13, 255
	sltu x15, x10, x9
	srli x13, x13, 8
	beqz x15, .Lelse
	
	mul x13, x13, x9
	add x13, x13, x10
	li x10, 256 + 8
	sub x10, x10, x9
	srli x10, x10, 4
	add x9, x9, x10
	j .Lendif

.Lelse:
	li x16, 256
	sub x16, x16, x9
	mul x13, x13, x16
	add x13, x13, x10
	sub x13, x13, x9
	addi x10, x9, 8
	srli x10, x10, 4
	sub x9, x9, x10

.Lendif:

	sb x9, 0(x14)
	
	mv x9, t0
	mv x14, t1
	mv x10, t2
	ret
