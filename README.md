# Upkr

Upkr is a simple general purpose lz packer designed to be used in the [MicroW8](https://github.com/exoticorn/microw8) platform.
The compressed format is losely based on [Shrinkler](https://github.com/askeksa/Shrinkler) with the main difference being that
Upkr doesn't differentiate between literals at odd or even addresses (by default) and that I went with rANS/rABS instead of a range coder.

Compression rate is on par with Shrinkler.

The differences compare to Shrinkler also makes it interesting on 8bit platforms. The z80 unpacker included in the release
is both about twice as fast and smaller than the Shrinkler unpacker.

## Inspirations:

* Ferris' blog about his [C64 intro packer](https://yupferris.github.io/blog/2020/08/31/c64-4k-intro-packer-deep-dive.html)
* [Shrinkler](https://github.com/askeksa/Shrinkler)
* Ryg's [sample rANS implementation](https://github.com/rygorous/ryg_rans)

## Unpackers

The release includes a reference c unpacker, as well as some optimized asm unpackers (arm and riscv). The unpckers in
c_unpacker and asm_unpackers unpack the default upkr compressed format. The z80_unpacker
is based on some variations to the compressed format. (Use `upkr --z80` to select those variations.)
The 16 bit dos unpacker also uses some variations. (`upkr --x86`)

### More unpackers outside this repository

* [Atari Lynx](https://github.com/42Bastian/new_bll/blob/master/demos/depacker/unupkr.asm)
* [Atari Jaguar](https://github.com/42Bastian/new_bjl/blob/main/exp/depacker/unupkr.js)
* [8080, R800](https://github.com/ivagorRetrocomp/DeUpkr)
* [6502](https://github.com/pfusik/upkr6502)

## Usage

```
  upkr [-l level(0-9)] [config options] <infile> [<outfile>]
  upkr -u [config options] <infile> [<outfile>]
  upkr --heatmap [config options] <infile> [<outfile>]
  upkr --margin [config options] <infile>

 -l, --level N       compression level 0-9
 -0, ..., -9         short form for setting compression level
 -d, --decompress    decompress infile
 --heatmap           calculate heatmap from compressed file
   --raw-cost        report raw cost of literals in heatmap
                     (the cost of literals is spread across all matches
                     that reference the literal by default.)
   --hexdump         print heatmap as colored hexdump
 --margin            calculate margin for overlapped unpacking of a packed file

When no infile is given, or the infile is '-', read from stdin.
When no outfile is given and reading from stdin, or when outfile is '-', write to stdout.

Config presets for specific unpackers:
 --z80               --big-endian-bitstream --invert-bit-encoding --simplified-prob-update -9
 --x86               --bitstream --invert-is-match-bit --invert-continue-value-bit --invert-new-offset-bit
 --x86b              --bitstream --invert-continue-value-bit --no-repeated-offsets -9

Config options (need to match when packing/unpacking):
 -b, --bitstream     bitstream mode
 -p, --parity N      use N (2/4) parity contexts
 -r, --reverse       reverse input & output

Config options to tailor output to specific optimized unpackers:
 --invert-is-match-bit
 --invert-new-offset-bit
 --invert-continue-value-bit
 --invert-bit-encoding
 --simplified-prob-update
 --big-endian-bitstream   (implies --bitstream)
 --no-repeated-offsets
 --eof-in-length
 --max-offset N
 --max-length N
```

## Heatmap

By default, the `--heatmap` flag writes out the heatmap data as a binary file. The heatmap file is
the same size as the unpacked data. Each byte can be interpreted like this:

```
is_literal = byte & 1; // whether the byte was encoded as a literal (as opposed to a match)
size_in_bits = 2.0 ** (((byte >> 1) - 64) / 8.0); // the size this byte takes up in the compressed data
```
