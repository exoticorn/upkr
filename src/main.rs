fn main() {
    let test_data = include_bytes!("../README.md");

    let packed = upkr::pack(test_data);
    dbg!((test_data.len(), packed.len()));

    let unpacked = upkr::unpack(&packed);
    dbg!(unpacked.len());
    assert!(test_data == unpacked.as_slice());
}
