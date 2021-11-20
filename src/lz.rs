use crate::context_state::ContextState;
use crate::range_coder::{RangeCoder, RangeDecoder};

pub struct LzCoder {
    contexts: ContextState,
    range_coder: RangeCoder,
    last_offset: usize,
}

impl LzCoder {
    pub fn new() -> LzCoder {
        LzCoder {
            contexts: ContextState::new(1 + 255 + 1 + 64 + 64),
            range_coder: RangeCoder::new(),
            last_offset: 0,
        }
    }

    pub fn encode_literal(&mut self, byte: u8) {
        self.bit(false, 0);
        let mut context_index = 1;
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1 != 0;
            self.bit(bit, context_index);
            context_index = (context_index << 1) | bit as usize;
        }
    }

    pub fn encode_match(&mut self, offset: usize, length: usize) {
        self.bit(true, 0);
        if offset != self.last_offset {
            self.last_offset = offset;
            self.bit(true, 256);
            self.length(offset + 1, 257);
        } else {
            self.bit(false, 256);
        }
        self.length(length, 257 + 64);
    }

    pub fn finish(mut self) -> Vec<u8> {
        self.bit(true, 0);
        self.bit(true, 256);
        self.length(1, 257);
        self.range_coder.finish()
    }

    pub fn last_offset(&self) -> usize {
        self.last_offset
    }

    fn length(&mut self, value: usize, context_start: usize) {
        assert!(value >= 1);
        let top_bit = usize::BITS - 1 - value.leading_zeros();
        let mut context_index = context_start;
        for i in 0..top_bit {
            self.bit(true, context_index);
            self.bit((value >> i) & 1 != 0, context_index + 1);
            context_index += 2;
        }
        self.bit(false, context_index);
    }

    fn bit(&mut self, b: bool, context_index: usize) {
        self.range_coder
            .encode_with_context(b, &mut self.contexts.context_mut(context_index));
    }
}

pub fn unpack(packed_data: &[u8]) -> Vec<u8> {
    let mut decoder = RangeDecoder::new(packed_data);
    let mut contexts = ContextState::new(1 + 255 + 1 + 64 + 64);
    let mut result = vec![];
    let mut offset = 0;

    fn decode_length(
        decoder: &mut RangeDecoder,
        contexts: &mut ContextState,
        mut context_index: usize,
    ) -> usize {
        let mut length = 0;
        let mut bit_pos = 0;
        while decoder.decode_with_context(&mut contexts.context_mut(context_index)) {
            length |= (decoder.decode_with_context(&mut contexts.context_mut(context_index + 1))
                as usize)
                << bit_pos;
            bit_pos += 1;
            context_index += 2;
        }
        length | (1 << bit_pos)
    }

    loop {
        if decoder.decode_with_context(&mut contexts.context_mut(0)) {
            if decoder.decode_with_context(&mut contexts.context_mut(256)) {
                offset = decode_length(&mut decoder, &mut contexts, 257) - 1;
                if offset == 0 {
                    break;
                }
            }
            let length = decode_length(&mut decoder, &mut contexts, 257 + 64);
            for _ in 0..length {
                result.push(result[result.len() - offset]);
            }
        } else {
            let mut context_index = 1;
            let mut byte = 0;
            for i in (0..8).rev() {
                let bit = decoder.decode_with_context(&mut contexts.context_mut(context_index));
                context_index = (context_index << 1) | bit as usize;
                byte |= (bit as u8) << i;
            }
            result.push(byte);
        }
    }

    result
}
