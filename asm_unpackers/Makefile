build/unpack_riscv64: ../c_unpacker/main.c unpack_riscv.S
	mkdir -p build
	riscv64-linux-gnu-gcc -g -static -o $@ $^

test_riscv64: build/unpack_riscv64
	qemu-riscv64 $< test_data.upk /tmp/out.bin
	cmp test_data.bin /tmp/out.bin

build/unpack_riscv64.o: unpack_riscv.S
	mkdir -p build
	riscv64-linux-gnu-gcc -c -o $@ $?

build/unpack_riscv64.bin: build/unpack_riscv64.o
	riscv64-linux-gnu-objcopy -O binary --only-section=.text $? $@

disas-riscv64: build/unpack_riscv64.o
	riscv64-linux-gnu-objdump -d $?

build/unpack_riscv32.o: unpack_riscv.S
	mkdir -p build
	riscv64-linux-gnu-gcc -march=rv32imc -mabi=ilp32 -c -o $@ $?
	
build/unpack_riscv32.bin: build/unpack_riscv32.o
	riscv64-linux-gnu-objcopy -O binary --only-section=.text $? $@

disas-riscv32: build/unpack_riscv32.o
	riscv64-linux-gnu-objdump -d $?

build/unpack_armv6m: ../c_unpacker/main.c unpack_armv6m.S
	mkdir -p build
	arm-linux-gnueabihf-gcc -g -static -o $@ $^

test_armv6m: build/unpack_armv6m
	qemu-arm $< test_data.upk /tmp/out.bin
	cmp test_data.bin /tmp/out.bin

build/unpack_armv6m.bin: unpack_armv6m.S
	mkdir -p build
	arm-none-eabi-gcc -march=armv6-m -c -o build/unpack_armv6m.o $?
	arm-none-eabi-objcopy -O binary --only-section=.text build/unpack_armv6m.o $@
	
build/unpack_arm32: ../c_unpacker/main.c unpack_arm32.S
	mkdir -p build
	arm-linux-gnueabihf-gcc -g -static -o $@ $^

test_arm32: build/unpack_arm32
	qemu-arm $< test_data.upk /tmp/out.bin
	cmp test_data.bin /tmp/out.bin

build/unpack_arm32.bin: unpack_arm32.S
	mkdir -p build
	arm-none-eabi-gcc -c -o build/unpack_arm32.o $?
	arm-none-eabi-objcopy -O binary --only-section=.text build/unpack_arm32.o $@
	
build/unpack_c: ../c_unpacker/main.c ../c_unpacker/unpack.c
	mkdir -p build
	gcc -g -o $@ $^

test_c: build/unpack_c
	$< test_data.upk /tmp/out.bin
	cmp test_data.bin /tmp/out.bin

sizes: build/unpack_armv6m.bin build/unpack_riscv64.bin build/unpack_riscv32.bin build/unpack_arm32.bin
	ls -l build/*.bin