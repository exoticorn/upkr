build/unpack_riscv64: ../c_unpacker/main.c ../c_unpacker/unpack.c
	mkdir -p build
	riscv64-linux-gnu-gcc -g -static -o $@ $^

test_riscv64: build/unpack_riscv64
	qemu-riscv64 $< test_data.upk /tmp/out.bin
	cmp test_data.bin /tmp/out.bin

build/unpack_armv6m: ../c_unpacker/main.c unpack_armv6m.S
	mkdir -p build
	arm-linux-gnueabihf-gcc -g -static -o $@ $^

test_armv6m: build/unpack_armv6m
	qemu-arm $< test_data.upk /tmp/out.bin
	cmp test_data.bin /tmp/out.bin

build/unpack_armv6m.bin: unpack_armv6m.S
	arm-none-eabi-gcc -march=armv6-m -c -o build/unpack_armv6m.o $?
	arm-none-eabi-objcopy -O binary --only-section=.text build/unpack_armv6m.o $@

sizes: build/unpack_armv6m.bin
	ls -l build/*.bin