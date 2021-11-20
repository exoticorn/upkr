# Upkr

Upkr is a simple general purpose lz packer designed to be used in the [MicroW8](https://github.com/exoticorn/microw8) platform.
The compressed format is base on [Shrinkler](https://github.com/askeksa/Shrinkler) with the main difference being that
Upkr doesn't differnetiate between literals at odd or even addresses.

At this point, Upkr should be considered unstable - the exact format isn't finalized yet and still subject to change
and only a very simple (but also very fast) greedy compressor is implemented. The compression ratio will be improved
with a more thourough lz parse in the future, although even in the current state is is already similar to the
DEFLATE compression algorithm.