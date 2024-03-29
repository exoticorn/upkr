typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned long u32;

u8* upkr_data_ptr;
u8 upkr_probs[1 + 255 + 1 + 2*32 + 2*32]; 
#ifdef UPKR_BITSTREAM
u16 upkr_state;
u8 upkr_current_byte;
int upkr_bits_left;
#else
u32 upkr_state;
#endif

int upkr_decode_bit(int context_index) {
#ifdef UPKR_BITSTREAM
    while(upkr_state < 32768) {
        if(upkr_bits_left == 0) {
            upkr_current_byte = *upkr_data_ptr++;
            upkr_bits_left = 8;
        }
        upkr_state = (upkr_state << 1) + (upkr_current_byte & 1);
        upkr_current_byte >>= 1;
        --upkr_bits_left;
    }
#else
    while(upkr_state < 4096) {
        upkr_state = (upkr_state << 8) | *upkr_data_ptr++;
    }
#endif
   
    int prob = upkr_probs[context_index];
    int bit = (upkr_state & 255) < prob ? 1 : 0;
    
    if(bit) {
        upkr_state = prob * (upkr_state >> 8) + (upkr_state & 255);
        prob += (256 - prob + 8) >> 4;
    } else {
        upkr_state = (256 - prob) * (upkr_state >> 8) + (upkr_state & 255) - prob;
        prob -= (prob + 8) >> 4;
    }
    upkr_probs[context_index] = prob;

    return bit;
}

int upkr_decode_length(int context_index) {
    int length = 0;
    int bit_pos = 0;
    while(upkr_decode_bit(context_index)) {
        length |= upkr_decode_bit(context_index + 1) << bit_pos++;
        context_index += 2;
    }
    return length | (1 << bit_pos);
}

void* upkr_unpack(void* destination, void* compressed_data) {
    upkr_data_ptr = (u8*)compressed_data;
    upkr_state = 0;
#ifdef UPKR_BITSTREAM
    upkr_bits_left = 0;
#endif
    for(int i = 0; i < sizeof(upkr_probs); ++i)
        upkr_probs[i] = 128;
    
    u8* write_ptr = (u8*)destination;
    
    int prev_was_match = 0;
    int offset = 0;
    for(;;) {
        if(upkr_decode_bit(0)) {
            if(prev_was_match || upkr_decode_bit(256)) {
                offset = upkr_decode_length(257) - 1;
                if(offset == 0) {
                    break;
                }
            }
            int length = upkr_decode_length(257 + 64);
            while(length--) {
                *write_ptr = write_ptr[-offset];
                ++write_ptr;
            }
            prev_was_match = 1;
        } else {
            int byte = 1;
            while(byte < 256) {
                int bit = upkr_decode_bit(byte);
                byte = (byte << 1) + bit;
            }
            *write_ptr++ = byte;
            prev_was_match = 0;
        }
    }
    
    return write_ptr;
}
