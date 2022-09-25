use crate::context_state::ContextState;
use crate::rans::{EntropyCoder, RansDecoder};
use crate::Config;

#[derive(Copy, Clone, Debug)]
pub enum Op {
    Literal(u8),
    Match { offset: u32, len: u32 },
}

impl Op {
    pub fn encode(&self, coder: &mut dyn EntropyCoder, state: &mut CoderState, config: &Config) {
        let literal_base = state.pos % state.parity_contexts * 256;
        match self {
            &Op::Literal(lit) => {
                encode_bit(coder, state, literal_base, !config.is_match_bit);
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
                encode_bit(coder, state, literal_base, config.is_match_bit);
                if !state.prev_was_match && !config.no_repeated_offsets {
                    encode_bit(
                        coder,
                        state,
                        256 * state.parity_contexts,
                        (offset != state.last_offset) == config.new_offset_bit,
                    );
                } else {
                    assert!(offset != state.last_offset || config.no_repeated_offsets);
                }
                if offset != state.last_offset || config.no_repeated_offsets {
                    encode_length(
                        coder,
                        state,
                        256 * state.parity_contexts + 1,
                        offset + 1,
                        config,
                    );
                    state.last_offset = offset;
                }
                encode_length(coder, state, 256 * state.parity_contexts + 65, len, config);
                state.prev_was_match = true;
                state.pos += len as usize;
            }
        }
    }
}

pub fn encode_eof(coder: &mut dyn EntropyCoder, state: &mut CoderState, config: &Config) {
    encode_bit(
        coder,
        state,
        state.pos % state.parity_contexts * 256,
        config.is_match_bit,
    );
    if !state.prev_was_match {
        encode_bit(
            coder,
            state,
            256 * state.parity_contexts,
            config.new_offset_bit,
        );
    }
    encode_length(coder, state, 256 * state.parity_contexts + 1, 1, config);
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
    config: &Config,
) {
    assert!(value >= 1);

    let mut context_index = context_start;
    while value >= 2 {
        encode_bit(coder, state, context_index, config.continue_value_bit);
        encode_bit(coder, state, context_index + 1, value & 1 != 0);
        context_index += 2;
        value >>= 1;
    }
    encode_bit(coder, state, context_index, !config.continue_value_bit);
}

#[derive(Clone)]
pub struct CoderState {
    contexts: ContextState,
    last_offset: u32,
    prev_was_match: bool,
    pos: usize,
    parity_contexts: usize,
}

impl CoderState {
    pub fn new(config: &Config) -> CoderState {
        CoderState {
            contexts: ContextState::new((1 + 255) * config.parity_contexts + 1 + 64 + 64, config),
            last_offset: 0,
            prev_was_match: false,
            pos: 0,
            parity_contexts: config.parity_contexts,
        }
    }

    pub fn last_offset(&self) -> u32 {
        self.last_offset
    }
}

pub fn unpack(packed_data: &[u8], config: &Config) -> Vec<u8> {
    let mut result = vec![];
    let _ = unpack_internal(Some(&mut result), packed_data, config);
    result
}

pub fn calculate_margin(packed_data: &[u8], config: &Config) -> isize {
    unpack_internal(None, packed_data, config)
}

pub fn unpack_internal(
    mut result: Option<&mut Vec<u8>>,
    packed_data: &[u8],
    config: &Config,
) -> isize {
    let mut decoder = RansDecoder::new(packed_data, &config);
    let mut contexts = ContextState::new((1 + 255) * config.parity_contexts + 1 + 64 + 64, &config);
    let mut offset = 0;
    let mut position = 0usize;
    let mut prev_was_match = false;
    let mut margin = 0isize;

    fn decode_length(
        decoder: &mut RansDecoder,
        contexts: &mut ContextState,
        mut context_index: usize,
        config: &Config,
    ) -> usize {
        let mut length = 0;
        let mut bit_pos = 0;
        while decoder.decode_with_context(&mut contexts.context_mut(context_index))
            == config.continue_value_bit
        {
            length |= (decoder.decode_with_context(&mut contexts.context_mut(context_index + 1))
                as usize)
                << bit_pos;
            bit_pos += 1;
            context_index += 2;
        }
        length | (1 << bit_pos)
    }

    loop {
        margin = margin.max(position as isize - decoder.pos() as isize);
        let literal_base = position % config.parity_contexts * 256;
        if decoder.decode_with_context(&mut contexts.context_mut(literal_base))
            == config.is_match_bit
        {
            if config.no_repeated_offsets
                || prev_was_match
                || decoder
                    .decode_with_context(&mut contexts.context_mut(256 * config.parity_contexts))
                    == config.new_offset_bit
            {
                offset = decode_length(
                    &mut decoder,
                    &mut contexts,
                    256 * config.parity_contexts + 1,
                    &config,
                ) - 1;
                if offset == 0 {
                    break;
                }
            }
            let length = decode_length(
                &mut decoder,
                &mut contexts,
                256 * config.parity_contexts + 65,
                &config,
            );
            if let Some(ref mut result) = result {
                for _ in 0..length {
                    result.push(result[result.len() - offset]);
                }
            }
            position += length;
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
            if let Some(ref mut result) = result {
                result.push(byte);
            }
            position += 1;
            prev_was_match = false;
        }
    }

    margin + decoder.pos() as isize - position as isize
}
