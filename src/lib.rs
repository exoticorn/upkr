mod context_state;
mod greedy_packer;
mod heatmap;
mod lz;
mod match_finder;
mod parsing_packer;
mod rans;

pub use heatmap::Heatmap;
pub use lz::{calculate_margin, create_heatmap, unpack, UnpackError};

pub type ProgressCallback<'a> = &'a mut dyn FnMut(usize);

#[derive(Debug)]
pub struct Config {
    pub use_bitstream: bool,
    pub parity_contexts: usize,

    pub invert_bit_encoding: bool,
    pub is_match_bit: bool,
    pub new_offset_bit: bool,
    pub continue_value_bit: bool,

    pub bitstream_is_big_endian: bool,
    pub simplified_prob_update: bool,

    pub no_repeated_offsets: bool,
    pub eof_in_length: bool,

    pub max_offset: usize,
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
    pub fn min_length(&self) -> usize {
        if self.eof_in_length {
            2
        } else {
            1
        }
    }
}

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

pub fn compressed_size(mut data: &[u8]) -> f32 {
    let mut state = 0;
    while state < 4096 {
        state = (state << 8) | data[0] as u32;
        data = &data[1..];
    }
    data.len() as f32 + (state as f32).log2() / 8.
}
