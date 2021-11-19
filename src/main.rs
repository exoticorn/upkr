mod context_state;
mod greedy_packer;
mod lz;
mod match_finder;
mod range_coder;

fn main() {
    let test_data = include_bytes!("../testcases/skipahead.wasm");

    let packed = greedy_packer::pack(test_data);
    dbg!((test_data.len(), packed.len()));

    let unpacked = lz::unpack(&packed);
    dbg!(unpacked.len());
    assert!(test_data == unpacked.as_slice());
}
