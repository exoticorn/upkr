use crate::context_state::ContextState;
use crate::rans::{EntropyCoder, RansDecoder};

#[derive(Copy, Clone, Debug)]
pub enum Op {
    Literal(u8),
    Match { offset: u32, len: u32 },
}

impl Op {
    pub fn encode(&self, coder: &mut dyn EntropyCoder, state: &mut CoderState) {
        match self {
            &Op::Literal(lit) => {
                encode_bit(coder, state, 0, false);
                let mut context_index = 1;
                for i in (0..8).rev() {
                    let bit = (lit >> i) & 1 != 0;
                    encode_bit(coder, state, context_index, bit);
                    context_index = (context_index << 1) | bit as usize;
                }
            }
            &Op::Match { offset, len } => {
                encode_bit(coder, state, 0, true);
                encode_bit(coder, state, 256, offset != state.last_offset);
                if offset != state.last_offset {
                    encode_length(coder, state, 257, offset + 1);
                    state.last_offset = offset;
                }
                encode_length(coder, state, 257 + 64, len);
            }
        }
    }
}

pub fn encode_eof(coder: &mut dyn EntropyCoder, state: &mut CoderState) {
    encode_bit(coder, state, 0, true);
    encode_bit(coder, state, 256, true);
    encode_length(coder, state, 257, 1);
}

fn encode_bit(
    coder: &mut dyn EntropyCoder,
    state: &mut CoderState,
    context_index: usize,
    bit: bool,
) {
    coder.encode_with_context(bit, &mut state.contexts.context_mut(context_index));
}

fn encode_length(
    coder: &mut dyn EntropyCoder,
    state: &mut CoderState,
    context_start: usize,
    value: u32,
) {
    assert!(value >= 1);
    let top_bit = u32::BITS - 1 - value.leading_zeros();
    let mut context_index = context_start;
    for i in 0..top_bit {
        encode_bit(coder, state, context_index, true);
        encode_bit(coder, state, context_index + 1, (value >> i) & 1 != 0);
        context_index += 2;
    }
    encode_bit(coder, state, context_index, false);
}

#[derive(Clone)]
pub struct CoderState {
    contexts: ContextState,
    last_offset: u32,
}

impl CoderState {
    pub fn new() -> CoderState {
        CoderState {
            contexts: ContextState::new(1 + 255 + 1 + 64 + 64),
            last_offset: 0,
        }
    }

    pub fn last_offset(&self) -> u32 {
        self.last_offset
    }
}

pub fn unpack(packed_data: &[u8]) -> Vec<u8> {
    let mut decoder = RansDecoder::new(packed_data);
    let mut contexts = ContextState::new(1 + 255 + 1 + 64 + 64);
    let mut result = vec![];
    let mut offset = 0;

    fn decode_length(
        decoder: &mut RansDecoder,
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
