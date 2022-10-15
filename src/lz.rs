use crate::context_state::ContextState;
use crate::rans::{EntropyCoder, RansDecoder};
use crate::Config;
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum Op {
    Literal(Vec<u8>),
    Match { offset: u32, len: u32 },
}

impl Op {
    pub fn encode(&self, coder: &mut dyn EntropyCoder, state: &mut CoderState, config: &Config) {
        match self {
            &Op::Literal(ref lit) => {
                assert!(state.prev_was_match);
                encode_length(
                    coder,
                    state,
                    256 + state.pos % state.parity_contexts * 320,
                    lit.len() as u32 + 1,
                    config,
                );
                for lit in lit {
                    let literal_base = state.pos % state.parity_contexts * 320;
                    let mut context_index = 1;
                    for i in (0..8).rev() {
                        let bit = (lit >> i) & 1 != 0;
                        encode_bit(coder, state, literal_base + context_index, bit);
                        context_index = (context_index << 1) | bit as usize;
                    }
                    state.pos += 1;
                }
                state.prev_was_match = false;
            }
            &Op::Match { offset, len } => {
                if state.prev_was_match {
                    encode_length(
                        coder,
                        state,
                        256 + state.pos % state.parity_contexts * 320,
                        1,
                        config,
                    );
                }
                let mut new_offset = true;
                if !state.prev_was_match && !config.no_repeated_offsets {
                    new_offset = offset != state.last_offset;
                    encode_bit(
                        coder,
                        state,
                        320 * state.parity_contexts,
                        new_offset == config.new_offset_bit,
                    );
                }
                assert!(offset as usize <= config.max_offset);
                if new_offset {
                    encode_length(
                        coder,
                        state,
                        320 * state.parity_contexts + 1,
                        offset + if config.eof_in_length { 0 } else { 1 },
                        config,
                    );
                    state.last_offset = offset;
                }
                assert!(len as usize >= config.min_length() && len as usize <= config.max_length);
                encode_length(coder, state, 320 * state.parity_contexts + 65, len, config);
                state.prev_was_match = true;
                state.pos += len as usize;
            }
        }
    }
}

pub fn encode_eof(coder: &mut dyn EntropyCoder, state: &mut CoderState, config: &Config) {
    if state.prev_was_match {
        encode_length(
            coder,
            state,
            256 + state.pos % state.parity_contexts * 320,
            1,
            config,
        );
    }
    if !state.prev_was_match && !config.no_repeated_offsets {
        encode_bit(
            coder,
            state,
            320 * state.parity_contexts,
            config.new_offset_bit ^ config.eof_in_length,
        );
    }
    if !config.eof_in_length || state.prev_was_match || config.no_repeated_offsets {
        encode_length(coder, state, 320 * state.parity_contexts + 1, 1, config);
    }
    if config.eof_in_length {
        encode_length(coder, state, 320 * state.parity_contexts + 65, 1, config);
    }
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
            contexts: ContextState::new((64 + 256) * config.parity_contexts + 1 + 64 + 64, config),
            last_offset: 0,
            prev_was_match: true,
            pos: 0,
            parity_contexts: config.parity_contexts,
        }
    }

    pub fn last_offset(&self) -> u32 {
        self.last_offset
    }
}

#[derive(Error, Debug)]
pub enum UnpackError {
    #[error("match offset out of range: {offset} > {position}")]
    OffsetOutOfRange { offset: usize, position: usize },
    #[error("Unpacked data over size limit: {size} > {limit}")]
    OverSize { size: usize, limit: usize },
    #[error("Unexpected end of input data")]
    UnexpectedEOF {
        #[from]
        source: crate::rans::UnexpectedEOF,
    },
    #[error("Overflow while reading value")]
    ValueOverflow,
}

pub fn unpack(
    packed_data: &[u8],
    config: &Config,
    max_size: usize,
) -> Result<Vec<u8>, UnpackError> {
    let mut result = vec![];
    let _ = unpack_internal(Some(&mut result), packed_data, config, max_size)?;
    Ok(result)
}

pub fn calculate_margin(packed_data: &[u8], config: &Config) -> Result<isize, UnpackError> {
    unpack_internal(None, packed_data, config, usize::MAX)
}

pub fn unpack_internal(
    mut result: Option<&mut Vec<u8>>,
    packed_data: &[u8],
    config: &Config,
    max_size: usize,
) -> Result<isize, UnpackError> {
    let mut decoder = RansDecoder::new(packed_data, &config);
    let mut contexts =
        ContextState::new((64 + 256) * config.parity_contexts + 1 + 64 + 64, &config);
    let mut offset = usize::MAX;
    let mut position = 0usize;
    let mut prev_was_match = false;
    let mut margin = 0isize;

    fn decode_length(
        decoder: &mut RansDecoder,
        contexts: &mut ContextState,
        mut context_index: usize,
        config: &Config,
    ) -> Result<usize, UnpackError> {
        let mut length = 0;
        let mut bit_pos = 0;
        while decoder.decode_with_context(&mut contexts.context_mut(context_index))?
            == config.continue_value_bit
        {
            length |= (decoder.decode_with_context(&mut contexts.context_mut(context_index + 1))?
                as usize)
                << bit_pos;
            bit_pos += 1;
            if bit_pos >= 32 {
                return Err(UnpackError::ValueOverflow);
            }
            context_index += 2;
        }
        Ok(length | (1 << bit_pos))
    }

    loop {
        margin = margin.max(position as isize - decoder.pos() as isize);
        let literal_length = decode_length(
            &mut decoder,
            &mut contexts,
            256 + position % config.parity_contexts * 320,
            config,
        )? - 1;
        for _ in 0..literal_length {
            let literal_base = position % config.parity_contexts * 320;
            let mut context_index = 1;
            let mut byte = 0;
            for i in (0..8).rev() {
                let bit = decoder
                    .decode_with_context(&mut contexts.context_mut(literal_base + context_index))?;
                context_index = (context_index << 1) | bit as usize;
                byte |= (bit as u8) << i;
            }
            if let Some(ref mut result) = result {
                if result.len() < max_size {
                    result.push(byte);
                }
            }
            position += 1;
            prev_was_match = false;
        }

        if config.no_repeated_offsets
            || prev_was_match
            || decoder
                .decode_with_context(&mut contexts.context_mut(320 * config.parity_contexts))?
                == config.new_offset_bit
        {
            offset = decode_length(
                &mut decoder,
                &mut contexts,
                320 * config.parity_contexts + 1,
                &config,
            )? - if config.eof_in_length { 0 } else { 1 };
            if offset == 0 {
                break;
            }
        }
        let length = decode_length(
            &mut decoder,
            &mut contexts,
            320 * config.parity_contexts + 65,
            &config,
        )?;
        if config.eof_in_length && length == 1 {
            break;
        }
        if offset > position {
            return Err(UnpackError::OffsetOutOfRange { offset, position });
        }
        if let Some(ref mut result) = result {
            for _ in 0..length {
                if result.len() < max_size {
                    result.push(result[result.len() - offset]);
                } else {
                    break;
                }
            }
        }
        position += length;
        prev_was_match = true;
    }

    if position > max_size {
        return Err(UnpackError::OverSize {
            size: position,
            limit: max_size,
        });
    }

    Ok(margin + decoder.pos() as isize - position as isize)
}
