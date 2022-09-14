all: unpack.bin example/example.sna

# binary is positioned from ORG 0, not usable, just assembling to verify the syntax
unpack.bin: unpack.asm
	sjasmplus --msg=war --lst --lstlab=sort --raw=unpack.bin unpack.asm

example/example.sna: unpack.asm example/example.asm
	cd example && sjasmplus --msg=war --lst --lstlab=sort example.asm

clean:
	$(RM) unpack.bin unpack.lst example/example.sna example/example.lst
