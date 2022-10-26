use crate::context_state::ContextState;
use crate::heatmap::Heatmap;
use crate::rans::{EntropyCoder, RansDecoder};
use crate::Config;
use thiserror::Error;

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
                let mut new_offset = true;
                if !state.prev_was_match && !config.no_repeated_offsets {
                    new_offset = offset != state.last_offset;
                    encode_bit(
                        coder,
                        state,
                        256 * state.parity_contexts,
                        new_offset == config.new_offset_bit,
                    );
                }
                assert!(offset as usize <= config.max_offset);
                if new_offset {
                    encode_length(
                        coder,
                        state,
                        256 * state.parity_contexts + 1,
                        offset + if config.eof_in_length { 0 } else { 1 },
                        config,
                    );
                    state.last_offset = offset;
                }
                assert!(len as usize >= config.min_length() && len as usize <= config.max_length);
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
    if !state.prev_was_match && !config.no_repeated_offsets {
        encode_bit(
            coder,
            state,
            256 * state.parity_contexts,
            config.new_offset_bit ^ config.eof_in_length,
        );
    }
    if !config.eof_in_length || state.prev_was_match || config.no_repeated_offsets {
        encode_length(coder, state, 256 * state.parity_contexts + 1, 1, config);
    }
    if config.eof_in_length {
        encode_length(coder, state, 256 * state.parity_contexts + 65, 1, config);
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

/// The error type for the uncompressing related functions
#[derive(Error, Debug)]
pub enum UnpackError {
    /// a match offset pointing beyond the start of the unpacked data was encountered
    #[error("match offset out of range: {offset} > {position}")]
    OffsetOutOfRange {
        /// the match offset
        offset: usize,
        /// the current position in the uncompressed stream
        position: usize,
    },
    /// The passed size limit was exceeded
    #[error("Unpacked data over size limit: {size} > {limit}")]
    OverSize {
        /// the size of the uncompressed data
        size: usize,
        /// the size limit passed into the function
        limit: usize,
    },
    /// The end of the packed data was reached without an encoded EOF marker
    #[error("Unexpected end of input data")]
    UnexpectedEOF {
        #[from]
        /// the underlying EOF error in the rANS decoder
        source: crate::rans::UnexpectedEOF,
    },
    /// An offset or length value was found that exceeded 32bit
    #[error("Overflow while reading value")]
    ValueOverflow,
}

/// Uncompress a piece of compressed data
///
/// Returns either the uncompressed data, or an `UnpackError`
///
/// # Parameters
///
/// - `packed_data`: the compressed data
/// - `config`: the exact compression format config used to compress the data
/// - `max_size`: the maximum size of uncompressed data to return. When this is exceeded,
///   `UnpackError::OverSize` is returned
pub fn unpack(
    packed_data: &[u8],
    config: &Config,
    max_size: usize,
) -> Result<Vec<u8>, UnpackError> {
    let mut result = vec![];
    let _ = unpack_internal(Some(&mut result), None, packed_data, config, max_size)?;
    Ok(result)
}

/// Calculates the minimum margin when overlapping buffers.
///
/// Returns the minimum margin needed between the end of the compressed data and the
/// end of the uncompressed data when overlapping the two buffers to save on RAM.
pub fn calculate_margin(packed_data: &[u8], config: &Config) -> Result<isize, UnpackError> {
    unpack_internal(None, None, packed_data, config, usize::MAX)
}

/// Calculates a `Heatmap` from compressed data.
///
/// # Parameters
///
/// - `packed_data`: the compressed data
/// - `config`: the exact compression format config used to compress the data
/// - `max_size`: the maximum size of the heatmap to return. When this is exceeded,
///   `UnpackError::OverSize` is returned
pub fn create_heatmap(
    packed_data: &[u8],
    config: &Config,
    max_size: usize,
) -> Result<Heatmap, UnpackError> {
    let mut heatmap = Heatmap::new();
    let _ = unpack_internal(None, Some(&mut heatmap), packed_data, config, max_size)?;
    Ok(heatmap)
}

fn unpack_internal(
    mut result: Option<&mut Vec<u8>>,
    mut heatmap: Option<&mut Heatmap>,
    packed_data: &[u8],
    config: &Config,
    max_size: usize,
) -> Result<isize, UnpackError> {
    let mut decoder = RansDecoder::new(packed_data, &config)?;
    let mut contexts = ContextState::new((1 + 255) * config.parity_contexts + 1 + 64 + 64, &config);
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
        let prev_decoder = decoder.clone();
        margin = margin.max(position as isize - decoder.pos() as isize);
        let literal_base = position % config.parity_contexts * 256;
        if decoder.decode_with_context(&mut contexts.context_mut(literal_base))?
            == config.is_match_bit
        {
            if config.no_repeated_offsets
                || prev_was_match
                || decoder
                    .decode_with_context(&mut contexts.context_mut(256 * config.parity_contexts))?
                    == config.new_offset_bit
            {
                offset = decode_length(
                    &mut decoder,
                    &mut contexts,
                    256 * config.parity_contexts + 1,
                    &config,
                )? - if config.eof_in_length { 0 } else { 1 };
                if offset == 0 {
                    break;
                }
            }
            let length = decode_length(
                &mut decoder,
                &mut contexts,
                256 * config.parity_contexts + 65,
                &config,
            )?;
            if config.eof_in_length && length == 1 {
                break;
            }
            if offset > position {
                return Err(UnpackError::OffsetOutOfRange { offset, position });
            }
            if let Some(ref mut heatmap) = heatmap {
                heatmap.add_match(offset, length, decoder.cost(&prev_decoder));
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
        } else {
            let mut context_index = 1;
            let mut byte = 0;
            for i in (0..8).rev() {
                let bit = decoder
                    .decode_with_context(&mut contexts.context_mut(literal_base + context_index))?;
                context_index = (context_index << 1) | bit as usize;
                byte |= (bit as u8) << i;
            }
            if let Some(ref mut heatmap) = heatmap {
                heatmap.add_literal(byte, decoder.cost(&prev_decoder));
            }
            if let Some(ref mut result) = result {
                if result.len() < max_size {
                    result.push(byte);
                }
            }
            position += 1;
            prev_was_match = false;
        }
    }

    if let Some(heatmap) = heatmap {
        heatmap.finish();
    }

    if position > max_size {
        return Err(UnpackError::OverSize {
            size: position,
            limit: max_size,
        });
    }

    Ok(margin + decoder.pos() as isize - position as isize)
}
