#ifndef UPKR_H_INCLUDED

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// input_buffer/input_size: input data to compress
// output_buffer/output_buffer_size: buffer to compress into
// compression_level: 0-9
// returns the size of the compressed data, even if it didn't fit into the output buffer
size_t upkr_compress(void* output_buffer, size_t output_buffer_size, void* input_buffer, size_t input_size, int compression_level);

// input_buffer/input_size: compressed data
// output_buffer/output_buffer_size: buffer to uncompress into
// return value:
//  >= 0 : size of uncompressed data, even if it didn't fit into the output buffer
//  < 0  : input data corrupt, unable to decompress
ptrdiff_t upkr_uncompress(void* output_buffer, size_t output_buffer_size, void* input_buffer, size_t input_size);

#ifdef __cplusplus
}
#endif
#endif