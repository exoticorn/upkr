16 bit DOS executable stubs
---------------------------

by pestis and TomCat

unpack_x86_16_DOS.asm:
  maximum compatibility, relocates unpacked code to normal start address
unpack_x86_16_DOS_no_relocation.asm:
  saves some bytes by not relocating, unpacked code needs to be assembled to
  start at 0x3FFE
unpack_x86_16_DOS_no_repeated_offset.asm:
  removes support for repeated offsets, potentially at the cost of some compression ratio.
  most likely only a win in very narrow circumstances around the 1kb mark