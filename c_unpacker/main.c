#include <stdio.h>
#include <stdlib.h>

void* upkr_unpack(void* destination, void* compressed_data);

int main(int argn, char** argv) {
  void* input_buffer = malloc(1024*1024);
  void* output_buffer = malloc(1024*1024);
  
  FILE* in_file = fopen(argv[1], "rb");
  int in_size = fread(input_buffer, 1, 1024*1024, in_file);
  fclose(in_file);
  
  printf("Compressed size: %d\n", in_size);
  
  void* end_ptr = upkr_unpack(output_buffer, input_buffer);
  int out_size = (char*)end_ptr - (char*)output_buffer;
  
  printf("Uncompressed size: %d\n", out_size);
  
  FILE* out_file = fopen(argv[2], "wb");
  fwrite(output_buffer, 1, out_size, out_file);
  fclose(out_file);
  
  return 0;
}
