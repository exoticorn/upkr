use crate::context_state::ContextState;
use crate::rans::{EntropyCoder, RansDecoder};

#[derive(Copy, Clone, Debug)]
pub enum Op {
    Literal(u8),
    Match { offset: u32, len: u32 },
}

impl Op {
    pub fn encode(&self, coder: &mut dyn EntropyCoder, state: &mut CoderState) {
        let base_context = 256 * (state.pos & 3);
        match self {
            &Op::Literal(lit) => {
                encode_bit(coder, state, base_context, false);
                let mut context_index = 1;
                for i in (0..8).rev() {
                    let bit = (lit >> i) & 1 != 0;
                    encode_bit(coder, state, base_context + context_index, bit);
                    context_index = (context_index << 1) | bit as usize;
                }
                state.pos += 1;
                state.prev_was_match = false;
            }
            &Op::Match { offset, len } => {
                encode_bit(coder, state, base_context, true);
                if !state.prev_was_match {
                    encode_bit(coder, state, 1024, offset != state.last_offset);
                } else {
                    assert!(offset != state.last_offset);
                }
                if offset != state.last_offset {
                    encode_length(coder, state, 1025, offset + 1);
                    state.last_offset = offset;
                }
                encode_length(coder, state, 1025 + 64, len);
                state.pos += len as usize;
                state.prev_was_match = true;
            }
        }
    }
}

pub fn encode_eof(coder: &mut dyn EntropyCoder, state: &mut CoderState) {
    encode_bit(coder, state, 256 * (state.pos & 3), true);
    if !state.prev_was_match {
        encode_bit(coder, state, 1024, true);
    }
    encode_length(coder, state, 1025, 1);
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
    mut value: u32,
) {
    assert!(value >= 1);

    let mut context_index = context_start;
    while value >= 2 {
        encode_bit(coder, state, context_index, true);
        encode_bit(coder, state, context_index + 1, value & 1 != 0);
        context_index += 2;
        value >>= 1;
    }
    encode_bit(coder, state, context_index, false);
}

#[derive(Clone)]
pub struct CoderState {
    contexts: ContextState,
    last_offset: u32,
    pos: usize,
    prev_was_match: bool,
}

impl CoderState {
    pub fn new() -> CoderState {
        CoderState {
            contexts: ContextState::new((1 + 255) * 4 + 1 + 64 + 64),
            last_offset: 0,
            pos: 0,
            prev_was_match: false,
        }
    }

    pub fn last_offset(&self) -> u32 {
        self.last_offset
    }
}

pub fn unpack(packed_data: &[u8], use_bitstream: bool) -> Vec<u8> {
    let mut decoder = RansDecoder::new(packed_data, use_bitstream);
    let mut contexts = ContextState::new((1 + 255) * 4 + 1 + 64 + 64);
    let mut result = vec![];
    let mut offset = 0;
    let mut prev_was_match = false;

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
        let base_context = 256 * (result.len() & 3);
        if decoder.decode_with_context(&mut contexts.context_mut(base_context)) {
            if prev_was_match || decoder.decode_with_context(&mut contexts.context_mut(1024)) {
                offset = decode_length(&mut decoder, &mut contexts, 1025) - 1;
                if offset == 0 {
                    break;
                }
            }
            let length = decode_length(&mut decoder, &mut contexts, 1025 + 64);
            for _ in 0..length {
                result.push(result[result.len() - offset]);
            }
            prev_was_match = true;
        } else {
            let mut context_index = 1;
            let mut byte = 0;
            for i in (0..8).rev() {
                let bit = decoder.decode_with_context(&mut contexts.context_mut(base_context + context_index));
                context_index = (context_index << 1) | bit as usize;
                byte |= (bit as u8) << i;
            }
            result.push(byte);
            prev_was_match = false;
        }
    }

    result
}
