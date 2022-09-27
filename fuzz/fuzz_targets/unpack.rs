#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = upkr::unpack(data, &upkr::Config::default(), 64 * 1024);
});
