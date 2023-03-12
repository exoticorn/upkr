upkr: upkr.c upkr.h target/release/libupkr.a
	gcc -O2 -Ltarget/release -o upkr upkr.c -lupkr -lm
	strip upkr

target/release/libupkr.a: cargo
	cargo build --release

.PHONY: cargo