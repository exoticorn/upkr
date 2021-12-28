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
