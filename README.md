# Upkr

Upkr is a simple general purpose lz packer designed to be used in the [MicroW8](https://github.com/exoticorn/microw8) platform.
The compressed format is losely based on [Shrinkler](https://github.com/askeksa/Shrinkler) with the main difference being that
Upkr doesn't differnetiate between literals at odd or even addresses and that I went with rANS/rABS instead of a range coder.

At this point, Upkr should still be considered unstable - the compressed format is not very likely to change but I still want
to keep that option open a little longer.

## Inspirations:

* Ferris' blog about his [C64 intro packer](https://yupferris.github.io/blog/2020/08/31/c64-4k-intro-packer-deep-dive.html)
* [Shrinkler](https://github.com/askeksa/Shrinkler)
* Ryg's [sample rANS implementation](https://github.com/rygorous/ryg_rans)