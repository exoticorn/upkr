;;; -*-asm-*-
;;; ukpr unpacker for Atari Jaguar RISC.

;;; lyxass syntax


; input:
;;; R20 : packed buffer
;;; R21 : output buffer
;;; r30 : return address
;;;
;;; Register usage (destroyed!)
;;; r0-r17,r20,r21
;;;

DST		REG 21
SRC		REG 20

	REGTOP 16
LR_save		REG 99
LR_save2	REG 99
GETBIT		REG 99
GETLENGTH	REG 99
LITERAL		REG 99
LOOP		REG 99
index		REG 99
bit_pos		REG 99
state		REG 99
prev_was_match	REG 99
offset		REG 99
prob		reg 99
byte		REG 99
PROBS		reg 99
tmp2		reg 2
tmp1		REG 1
tmp0		REG 0

	REGMAP

upkr_probs	equ $200

SIZEOF_PROBS	EQU 1+255+1+2*32+2*32

unupkr::
	move	LR,LR_save
	moveq	#0,tmp0
	movei	#upkr_probs,PROBS
	bset	#7,tmp0
	movei	#SIZEOF_PROBS,tmp2
	move	PROBS,tmp1
.init	storeb	tmp0,(tmp1)
	subq	#1,tmp2
	jr	pl,.init
	addq	#1,tmp1

	moveq	#0,offset
	moveq	#0,state
	movei	#getlength,GETLENGTH
	movei	#getbit,GETBIT
.looppc	move	PC,LOOP
	addq	#.loop-.looppc,LOOP
	move	pc,LITERAL
	jr	.start
	addq	#6,LITERAL

.literal
	moveq	#1,byte
	move	pc,LR
	jr	.into
	addq	#6,LR		; LR = .getbit
.getbit
	addc	byte,byte
.into
	btst	#8,byte
	jump	eq,(GETBIT)
	move	byte,index

	storeb	byte,(DST)
	addq	#1,DST
.start
	moveq	#0,prev_was_match

.loop
	moveq	#0,index
	BL	(GETBIT)
	jump	cc,(LITERAL)
	addq	#14,LR
	cmpq	#1,prev_was_match
	jr	eq,.newoff
	shlq	#8,r0
	jump	(GETBIT)
	move	r0,index
	jr	cc,.oldoff
	shlq	#8,r0
.newoff
	addq	#1,r0		; r0 = 257
	BL	(GETLENGTH)
	subq	#1,r0
	jump	eq,(LR_save)
	move	r0,offset

.oldoff
	movei	#257+64,r0
	BL	(GETLENGTH)

	move	DST,r1
	sub	offset,r1
.cpymatch1
	loadb	(r1),r2
	subq	#1,r0
	addqt	#1,r1
	storeb	r2,(DST)
	jr	ne,.cpymatch1
	addq	#1,DST

	jump	(LOOP)
	moveq	#1,prev_was_match

getlength:
	move	LR,LR_save2
	moveq	#0,byte
	move	r0,index
	moveq	#0,bit_pos
	move	pc,LR
	jump	(GETBIT)
	addq	#6,LR
.gl
	jr	cc,.exit
	addq	#8,LR		; => return to "sh ..."
	jump	(GETBIT)
	nop
	sh	bit_pos,r0
	subq	#1,bit_pos	; sh < 0 => shift left!
	or	r0,byte
	jump	(GETBIT)
	subq	#8,LR
.exit
	moveq	#1,r0
	sh	bit_pos,r0
	jump	(LR_save2)
	or	byte,r0

.newbyte:
	loadb	(SRC),r2
	shlq	#8,state
	addq	#1,SRC
	or	r2,state
getbit
	move	state,r2
	move	PROBS,r1
	add	index,r1		; r1 = &probs[index]
	shrq	#12,r2
	loadb	(r1),prob
	jr	eq,.newbyte
	move	state,r2
	move	state,r0
	shlq	#24,r2
	shrq	#8,r0		; sh
	shrq	#24,r2		; sl
	cmp	prob,r2
	addqt	#1,index
	jr	cs,.one
	mult	prob,r0

	;; state -= ((state >> 8) + 1)*prob
	;; prob -= (prob+8)>>4
	move	prob,r2
	add	prob,r0
	addq	#8,r2
	sub	r0,state
	shrq	#4,r2
	moveq	#0,r0
	jr	.ret
	sub	r2,prob

.one
	;; state = (state >> 8)*prob+(state & 0xff)
	;; prob += (256 + 8 - prob) >> 4
	move	r2,state
	movei	#256+8,r2
	add	r0,state
	sub	prob,r2		; 256-prob+8
	shrq	#4,r2
	add	r2,prob

	moveq	#3,r0
.ret
	storeb	prob,(r1)
	jump	(LR)
	shrq	#1,r0		; C = 0, r0 = 1
