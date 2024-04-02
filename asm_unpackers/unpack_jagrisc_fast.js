;;; -*-asm-*-
;;; ukpr unpacker for Atari Jaguar RISC. (quick version)

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

	REGTOP 17
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
ndata		reg 99
PROBS		reg 99
tmp2		reg 2
tmp1		REG 1
tmp0		REG 0

	REGMAP

upkr_probs	equ $200

SIZEOF_PROBS	EQU 1+255+1+2*32+2*32

unupkr::
	move	LR,LR_save
	movei	#$80808080,tmp0
	movei	#upkr_probs,PROBS
	movei	#SIZEOF_PROBS,tmp2
	move	PROBS,tmp1
.init	store	tmp0,(tmp1)
	subq	#4,tmp2
	jr	pl,.init
	addq	#4,tmp1

	loadb	(SRC),ndata
	addq	#1,SRC
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
	move	r0,offset
	jump	eq,(LR_save)
	nop
.oldoff
	movei	#257+64,r0
	BL	(GETLENGTH)

	move	DST,r2
	move	DST,r1
	or	offset,r2
	btst	#0,r2
	moveq	#1,prev_was_match
	jr	ne,.cpymatch1
	sub	offset,r1
.cpymatch2
	loadw	(r1),r2
	addqt	#2,r1
	subq	#2,r0
	storew	r2,(DST)
	jump	eq,(LOOP)
	addqt	#2,DST
	jr	pl,.cpymatch2
	nop
	jump	(LOOP)
	subq	#1,DST

.cpymatch1
	loadb	(r1),r2
	subq	#1,r0
	addqt	#1,r1
	storeb	r2,(DST)
	jr	ne,.cpymatch1
	addq	#1,DST

	jump	(LOOP)
//->	nop

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
	move	ndata,r2
	shlq	#8,state
	loadb	(SRC),ndata
	or	r2,state
	addq	#1,SRC
	move	state,r2
	shrq	#12,r2
	jr	ne,.done
	move	state,r2
	jr	.newbyte
getbit
	move	state,r2
	move	PROBS,r1
	add	index,r1		; r1 = &probs[index]
	shrq	#12,r2
	loadb	(r1),prob
	jr	eq,.newbyte
	move	state,r2
.done
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
	sub	r2,prob
	shrq	#1,r0		; C = 0, r0 = 0
	jump	(LR)
	storeb	prob,(r1)

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
	storeb	prob,(r1)
	jump	(LR)
	shrq	#1,r0		; C = 0, r0 = 1
