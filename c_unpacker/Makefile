all: unpack unpack_bitstream

unpack: main.c unpack.c
	cc -O2 -o unpack main.c unpack.c

unpack_bitstream: main.c unpack.c
	cc -O2 -D UPKR_BITSTREAM -o unpack_bitstream main.c unpack.c
	
unpack_debug: main.c unpack.c
	cc -g -o unpack_debug main.c unpack.c
