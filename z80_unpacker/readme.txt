Z80 asm implementation of C unpacker, code-size focused (not performance).

**ONLY BITSTREAM** variant is currently supported, make sure to use "-b" in packer.

The project is expected to further evolve, including possible changes to binary format, this is
initial version of Z80 unpacker to explore if/how it works and how it can be improved further.

(copy full packer+depacker source to your project if you plan to use it, as future revisions
may be incompatible with files you will produce with current version)

Asm syntax is z00m's sjasmplus: https://github.com/z00m128/sjasmplus

Backward direction unpacker added as compile-time option, see example for both forward/backward
depacker in action.

The packed/unpacked data-overlap has to be tested per-case, in worst case the packed data
may need even more than 7 bytes to unpack final byte, but usually 1-4 bytes may suffice.

TODO:
- build bigger corpus of test data to benchmark future changes in algorithm/format (example and zx48.rom was used to do initial tests)
- maybe try to beat double-loop `decode_number` with different encoding format
- (@ped7g) Z80N version of unpacker for ZX Next devs
- (@exoticorn) add Z80 specific packer (to avoid confusion with original MicroW8 variant), and land it all to master branch, maybe in "z80" directory or something? (and overall decide how to organise+merge this upstream into main repo)
- (@exoticorn) add to packer output with possible packed/unpacked region overlap

DONE:
* review non-bitstream variant, if it's feasible to try to implement it with Z80
    - Ped7g: IMHO nope, the 12b x 8b MUL code would probably quickly cancel any gains from the simpler state update
* review first implementation to identify weak spots where the implementation can be shorter+faster
with acceptable small changes to the format
    - Ped7g: the decode_bit settled down and now doesn't feel so confused and redundant, the code seems pretty on point to me, no obvious simplification from format change
    - Ped7g: the decode_number double-loop is surprisingly resilient, especially in terms of code size I failed to beat it, speed wise only negligible gains
