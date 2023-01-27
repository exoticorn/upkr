#![deny(missing_docs)]

//! Compression and decompression of the upkr format and variants.
//!
//! Upkr is a compression format initially designed for the MicroW8 fantasy console,
//! with design goals being a competitive compression ratio, reasonable fast
//! decompression, low memory overhead and very small decompression code
//! when handoptimized in assembler. (An optimized DOS execuable decompressor is <140 bytes.)

mod context_state;
mod greedy_packer;
mod heatmap;
mod lz;
mod match_finder;
mod parsing_packer;
mod rans;

pub use heatmap::Heatmap;
pub use lz::{calculate_margin, create_heatmap, unpack, UnpackError};

/// The type of a callback function to be given to the `pack` function.
///
/// It will be periodically called with the number of bytes of the input already processed.
pub type ProgressCallback<'a> = &'a mut dyn FnMut(usize);

/// A configuration of which compression format variation to use.
///
/// Use `Config::default()` for the standard upkr format.
///
/// Compression format variants exist to help with micro-optimizations in uncompression
/// code on specific platforms.

#[derive(Debug)]
pub struct Config {
    /// Shift in bits from a bitstream into the rANS state, rather than whole bytes.
    /// This decreases the size of the rNAS state to 16 bits which is very useful on
    /// 8 bit platforms.
    pub use_bitstream: bool,
    /// The number of parity contexts (usually 1, 2 or 4). This can improve compression
    /// on data that consists of regular groups of 2 or 4 bytes. One example is 32bit ARM
    /// code, where each instruction is 4 bytes, so `parity_contexts = 4` improves compression
    /// quite a bit. Defaults to `1`.
    pub parity_contexts: usize,

    /// Invert the encoding of bits in the rANS coder. `bit = state_lo >= prob` instead of
    /// `bit = state_lo < prob`.
    pub invert_bit_encoding: bool,
    /// The boolean value which encodes a match. Defaults to `true`.
    pub is_match_bit: bool,
    /// The boolean value which encodes a new offset (rather than re-using the previous offset).
    /// Defaults to `true`.
    pub new_offset_bit: bool,
    /// The boolean value which encodes that there are more bits comming for length/offset values.
    /// Defaults to `true`.
    pub continue_value_bit: bool,

    /// Reverses the bits in the bitstream.
    pub bitstream_is_big_endian: bool,
    /// A slightly less accurate, but slightly simpler variation of the prob update in the
    /// rANS coder, Used for the z80 uncompressor.
    pub simplified_prob_update: bool,

    /// Disables support for re-using the last offset in the compression format.
    /// This might save a few bytes when working with very small data.
    pub no_repeated_offsets: bool,
    /// Standard upkr encodes the EOF marker in the offset. This encodes it in the match length
    /// instead.
    pub eof_in_length: bool,

    /// The maximum match offset value to encode when compressing.
    pub max_offset: usize,
    /// The maximum match length value to encode when compressing.
    pub max_length: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            use_bitstream: false,
            parity_contexts: 1,

            invert_bit_encoding: false,
            is_match_bit: true,
            new_offset_bit: true,
            continue_value_bit: true,

            bitstream_is_big_endian: false,
            simplified_prob_update: false,

            no_repeated_offsets: false,
            eof_in_length: false,

            max_offset: usize::MAX,
            max_length: usize::MAX,
        }
    }
}

impl Config {
    fn min_length(&self) -> usize {
        if self.eof_in_length {
            2
        } else {
            1
        }
    }
}

/// Compresses the given data.
///
/// # Arguments
/// - `data`: The data to compress
/// - `level`: The compression level (0-9). Increasing the level by one roughly halves the
///   compression speed.
/// - `config`: The compression format variant to use.
/// - `progress_callback`: An optional callback which will periodically be called with
///   the number of bytes already processed.
///
/// # Example
/// ```rust
/// let compressed_data = upkr::pack(b"Hello, World! Yellow world!", 0, &upkr::Config::default(), None);
/// assert!(compressed_data.len() < 27);
/// ```
pub fn pack(
    data: &[u8],
    level: u8,
    config: &Config,
    progress_callback: Option<ProgressCallback>,
) -> Vec<u8> {
    if level == 0 {
        greedy_packer::pack(data, config, progress_callback)
    } else {
        parsing_packer::pack(data, level, config, progress_callback)
    }
}

/// Estimate the exact (fractional) size of upkr compressed data.
///
/// Note that this currently does NOT work for the bitstream variant.
pub fn compressed_size(mut data: &[u8]) -> f32 {
    let mut state = 0;
    while state < 4096 {
        state = (state << 8) | data[0] as u32;
        data = &data[1..];
    }
    data.len() as f32 + (state as f32).log2() / 8.
}
