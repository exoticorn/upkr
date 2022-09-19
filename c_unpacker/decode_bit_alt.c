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
        prob = 256 - prob;
    }
    upkr_state -= prob * ((upkr_state >> 8) + (bit ^ 1));
    prob -= (prob + 8) >> 4;
    if(bit) {
        prob = -prob;
    }
    upkr_probs[context_index] = prob;

    return bit;
}

