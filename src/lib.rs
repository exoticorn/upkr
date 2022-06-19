mod context_state;
mod greedy_packer;
mod lz;
mod match_finder;
mod parsing_packer;
mod rans;

pub use lz::unpack;

pub type ProgressCallback<'a> = &'a mut dyn FnMut(usize);

pub fn pack(
    data: &[u8],
    level: u8,
    use_bitstream: bool,
    progress_callback: Option<ProgressCallback>,
) -> Vec<u8> {
    if level == 0 {
        greedy_packer::pack(data, use_bitstream, progress_callback)
    } else {
        parsing_packer::pack(data, level, use_bitstream, progress_callback)
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
