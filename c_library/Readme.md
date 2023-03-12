This is a simple example of compiling upkr to a library that can be linked in a
c program. It consists of a small rust crate which implements the c api and
compiles to a static library and a matching c header file. As is, the rust
crate offers two simple functions to compress/uncompress data with the default
upkr config.

The provided makefile will only work on linux. Building the example upkr.c on
other platforms is left as an exercise for the reader ;)

On Windows you might have to make sure to install and use the correct rust
toolchain version (mingw vs. msvc) to match your c compiler.