#include "upkr.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

int main(int argc, char** argv) {
    if(argc < 2) {
        fprintf(stdout, "Usage:\n  upkr [compress] [-0 .. -9] <file> [<out-file>]\n  upkr [uncompress] <file> [<out-file>]\n");
        return 1;
    }

    int argi = 1;
    int uncompress = 0;
    int compression_level = 4;
    if(strcmp(argv[argi], "compress") == 0) {
        ++argi;
    } else if(strcmp(argv[argi], "uncompress") == 0) {
        uncompress = 1;
        ++argi;
    }

    if(argi < argc && argv[argi][0] == '-') {
        compression_level = atoi(argv[argi] + 1);
        ++argi;
    }

    if(argi == argc) {
        fprintf(stdout, "intput filename missing\n");
        return 1;
    }

    const char* input_name = argv[argi++];
    char* output_name;
    if(argi < argc) {
        output_name = argv[argi];
    } else {
        output_name = malloc(strlen(input_name) + 5);
        strcpy(output_name, input_name);
        strcat(output_name, uncompress ? ".unp" : ".upk");
    }

    FILE* file = fopen(input_name, "rb");
    if(file == 0) {
        fprintf(stdout, "failed to open input file '%s'\n", file);
        return 1;
    }
    fseek(file, 0, SEEK_END);
    long input_size = ftell(file);
    rewind(file);

    char* input_buffer = (char*)malloc(input_size);
    long offset = 0;
    while(offset < input_size) {
        long read_size = fread(input_buffer + offset, 1, input_size - offset, file);
        if(read_size <= 0) {
            fprintf(stdout, "error reading input file\n");
            return 1;
        }
        offset += read_size;
    }
    fclose(file);

    long output_buffer_size = input_size * 8;
    long output_size;
    char* output_buffer = (char*)malloc(output_buffer_size);
    for(;;) {
        if(uncompress) {
            output_size = upkr_uncompress(output_buffer, output_buffer_size, input_buffer, input_size);
        } else {
            output_size = upkr_compress(output_buffer, output_buffer_size, input_buffer, input_size, compression_level);
        }
        if(output_size < 0) {
            return 1;
        }
        if(output_size <= output_buffer_size) {
            break;
        }
        output_buffer = (char*)realloc(output_buffer, output_size);
        output_buffer_size = output_size;
    }

    file = fopen(output_name, "wb");
    if(file == 0) {
        fprintf(stdout, "failed to open output file '%s'\n", output_name);
        return 1;
    }
    offset = 0;
    while(offset < output_size) {
        long written_size = fwrite(output_buffer + offset, 1, output_size - offset, file);
        if(written_size <= 0) {
            fprintf(stdout, "error writing output file\n");
            return 1;
        }
        offset += written_size;
    }
    fclose(file);

    return 0;
}