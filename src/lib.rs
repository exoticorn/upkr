mod context_state;
mod greedy_packer;
mod lz;
mod match_finder;
mod rans;
mod parsing_packer;

pub use lz::unpack;

pub type ProgressCallback<'a> = &'a mut dyn FnMut(usize);

pub fn pack(data: &[u8], level: u8, progress_callback: Option<ProgressCallback>) -> Vec<u8> {
    if level == 0 {
        greedy_packer::pack(data, progress_callback)
    } else {
        parsing_packer::pack(data, level, progress_callback)
    }
}
