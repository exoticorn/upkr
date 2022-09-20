use crate::context_state::ContextState;
use crate::rans::{EntropyCoder, RansDecoder};

#[derive(Copy, Clone, Debug)]
pub enum Op {
    Literal(u8),
    Match { offset: u32, len: u32 },
}

impl Op {
    pub fn encode(&self, coder: &mut dyn EntropyCoder, state: &mut CoderState) {
        let literal_base = state.pos % state.parity_contexts * 256;
        match self {
            &Op::Literal(lit) => {
                encode_bit(coder, state, literal_base, false);
                let mut context_index = 1;
                for i in (0..8).rev() {
                    let bit = (lit >> i) & 1 != 0;
                    encode_bit(coder, state, literal_base + context_index, bit);
                    context_index = (context_index << 1) | bit as usize;
                }
                state.prev_was_match = false;
                state.pos += 1;
            }
            &Op::Match { offset, len } => {
                encode_bit(coder, state, literal_base, true);
                if !state.prev_was_match {
                    encode_bit(
                        coder,
                        state,
                        256 * state.parity_contexts,
                        offset != state.last_offset,
                    );
                } else {
                    assert!(offset != state.last_offset);
                }
                if offset != state.last_offset {
                    encode_length(coder, state, 256 * state.parity_contexts + 1, offset + 1);
                    state.last_offset = offset;
                }
                encode_length(coder, state, 256 * state.parity_contexts + 65, len);
                state.prev_was_match = true;
                state.pos += len as usize;
            }
        }
    }
}

pub fn encode_eof(coder: &mut dyn EntropyCoder, state: &mut CoderState) {
    encode_bit(coder, state, state.pos % state.parity_contexts * 256, true);
    if !state.prev_was_match {
        encode_bit(coder, state, 256 * state.parity_contexts, true);
    }
    encode_length(coder, state, 256 * state.parity_contexts + 1, 1);
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
    parity_contexts: usize,
    last_offset: u32,
    prev_was_match: bool,
    pos: usize,
}

impl CoderState {
    pub fn new(parity_contexts: usize) -> CoderState {
        CoderState {
            contexts: ContextState::new((1 + 255) * parity_contexts + 1 + 64 + 64),
            last_offset: 0,
            parity_contexts,
            prev_was_match: false,
            pos: 0,
        }
    }

    pub fn last_offset(&self) -> u32 {
        self.last_offset
    }
}

pub fn unpack(packed_data: &[u8], use_bitstream: bool, parity_contexts: usize) -> Vec<u8> {
    let mut decoder = RansDecoder::new(packed_data, use_bitstream);
    let mut contexts = ContextState::new((1 + 255) * parity_contexts + 1 + 64 + 64);
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
        let literal_base = result.len() % parity_contexts * 256;
        if decoder.decode_with_context(&mut contexts.context_mut(literal_base)) {
            if prev_was_match
                || decoder.decode_with_context(&mut contexts.context_mut(256 * parity_contexts))
            {
                offset = decode_length(&mut decoder, &mut contexts, 256 * parity_contexts + 1) - 1;
                if offset == 0 {
                    break;
                }
            }
            let length = decode_length(&mut decoder, &mut contexts, 256 * parity_contexts + 65);
            for _ in 0..length {
                result.push(result[result.len() - offset]);
            }
            prev_was_match = true;
        } else {
            let mut context_index = 1;
            let mut byte = 0;
            for i in (0..8).rev() {
                let bit = decoder
                    .decode_with_context(&mut contexts.context_mut(literal_base + context_index));
                context_index = (context_index << 1) | bit as usize;
                byte |= (bit as u8) << i;
            }
            result.push(byte);
            prev_was_match = false;
        }
    }

    result
}
