/*
    A simple C unpacker for upkr compressed data.

    This implements two variants, selected by the UPKR_BITSTREAM define:
    - normal: faster and smaller on modern hardware as whole bytes are shifted into
              the rANS state at a time, but requires 20bits for the state
    - bitstream: only single bits are shifted into the rANS state at a time
                 which allows the state to always fit in 16bits which is a boon
                 on very old CPUs.
    The encoder and decoder need to be configured to use the same varianet.

    upkr compressed data is a rANS byte-/bit-stream encoding a series of literal
    byte values and back-references as probability encoded bits.

    upkr_decode_bit reads one bit from the rANS stream, taking a probability context
    as parameter. The probability context is a byte estimating the probability of
    a bit encoded in this context being set. It is updated by upkr_decode_bit
    after each decoded bit to reflect the observed past frequencies of on/off bits.

    There are a number of different contexts used in the compressed format. The order in the
    upkr_probs array is arbitrary, the only requirement for the unpacker is that all bits
    that shared the same context while encoding also share the same context while decoding.
    The contexts are:
    - is match
    - has offset
    - literal bit N (0-7) with already decoded highest bits of literal == M (255 total)
    - offset bit N (one less than max offset bits)
    - has offset bit N (max offset bits)
    - length bit N (one less then max length bits)
    - has length bit N (max length bits)

    Literal bytes are encoded from highest to lowest bit, with the bit position and
    the already decoded bits as context.

    Offst and Length are encoded in an interlaced variant of elias gamma coding. They
    are encoded from lowest to highest bits. For each bit, first one bit is read in the
    "has offset/length bit N)". If this is set, offset/length bit N is read in it's context
    and the decoding continues with the next bit. If the "has bit N" is read as false, a
    fixed 1 bit is added as the top bit at this position.

    The highlevel decode loop then looks like this:
    loop:
        if read_bit(IS_MATCH):
            if prev_was_match || read_bit(HAS_OFFSET):
                offset = read_length_or_offset(OFFSET) - 1
                if offset == 0:
                    break
            length = read_length_or_offset(LENGTH)
            copy_bytes_from_offset(length, offset)
        else:
            read_and_push(literal)
*/

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
    // shift in single bits until rANS state is >= 32768
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
    // shift in a full byte until rANS state is >= 4096
    while(upkr_state < 4096) {
        upkr_state = (upkr_state << 8) | *upkr_data_ptr++;
    }
#endif
   
    int prob = upkr_probs[context_index];
    int bit = (upkr_state & 255) < prob ? 1 : 0;
    
    // rANS state and context probability update
    // for the later, add 1/16th (rounded) of difference from either 0 or 256
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
    // all contexts are initialized to 128 = equal probability of 0 and 1
    for(int i = 0; i < sizeof(upkr_probs); ++i)
        upkr_probs[i] = 128;
    
    u8* write_ptr = (u8*)destination;
    
    int prev_was_match = 0;
    int offset = 0;
    for(;;) {
        // is match
        if(upkr_decode_bit(0)) {
            // has offset
            if(prev_was_match || upkr_decode_bit(256)) {
                offset = upkr_decode_length(257) - 1;
                if(offset == 0) {
                    // a 0 offset signals the end of the compressed data
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
            // byte contains the previously read bits and indicates the number of
            // read bits by the set top bit. Therefore it can be directly used as the
            // context index. The set top bit ends up at bit position 8 and is not stored.
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
