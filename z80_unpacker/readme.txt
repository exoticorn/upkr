Z80 asm implementation of C unpacker, code-size focused (not performance).

**ONLY BITSTREAM** variant is currently supported, make sure to use "-b" in packer.

The project is expected to further evolve, including possible changes to binary format, this is
initial version of Z80 unpacker to explore if/how it works and how it can be improved further.

(copy full packer+depacker source to your project if you plan to use it, as future revisions
may be incompatible with files you will produce with current version)

Asm syntax is z00m's sjasmplus: https://github.com/z00m128/sjasmplus

TODO:
- build base corpus of test data to benchmark future changes in algorithm/format
- review first implementation to identify weak spots where the implementation can be shorter+faster
with acceptable small changes to the format
- review non-bitstream variant, if it's feasible to try to implement it with Z80
- (@ped7g) Z80N version of unpacker for ZX Next devs
- (@exoticorn) add Z80 specific packer (to avoid confusion with original MicroW8 variant), and land it all to master branch, maybe in "z80" directory or something? (and overall decide how to organise+merge this upstream into main repo)
