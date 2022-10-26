VERSION := $(shell cargo run --release -- --version)

all: clean upkr-linux-$(VERSION).tgz upkr-windows-$(VERSION).zip

clean:
	rm -rf upkr-linux
	rm -f upkr-linux*.tgz
	rm -rf upkr-windows
	rm -f upkr-windows*.zip

upkr-linux-$(VERSION).tgz: upkr-linux/upkr PHONY
	cp ../README.md upkr-linux
	cd .. && git archive HEAD c_unpacker | tar -xC release/upkr-linux
	cd .. && git archive HEAD z80_unpacker | tar -xC release/upkr-linux
	cd .. && git archive HEAD asm_unpackers | tar -xC release/upkr-linux
	tar czf $@ upkr-linux

upkr-windows-$(VERSION).zip: upkr-windows/upkr.exe PHONY
	cp ../README.md upkr-windows/
	cd .. && git archive HEAD c_unpacker | tar -xC release/upkr-windows
	cd .. && git archive HEAD z80_unpacker | tar -xC release/upkr-windows
	cd .. && git archive HEAD asm_unpackers | tar -xC release/upkr-windows
	zip -r -9 $@ upkr-windows

upkr-linux/upkr:
	cargo build --target x86_64-unknown-linux-musl --release -F terminal
	mkdir -p upkr-linux
	cp ../target/x86_64-unknown-linux-musl/release/upkr upkr-linux/

upkr-windows/upkr.exe:
	cargo build --target x86_64-pc-windows-gnu --release -F terminal
	mkdir -p upkr-windows
	cp ../target/x86_64-pc-windows-gnu/release/upkr.exe upkr-windows/
	
PHONY: