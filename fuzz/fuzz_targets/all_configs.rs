#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut config = upkr::Config::default();
    let mut level = 1;
    let mut data = data;
    if data.len() > 2 {
        let flags1 = data[0];
        let flags2 = data[1];
        data = &data[2..];
        config.use_bitstream = (flags1 & 1) != 0;
        config.parity_contexts = if (flags1 & 2) == 0 { 1 } else { 2 };
        config.invert_bit_encoding = (flags1 & 4) != 0;
        config.is_match_bit = (flags1 & 8) != 0;
        config.new_offset_bit = (flags1 & 16) != 0;
        config.continue_value_bit = (flags1 & 32) != 0;
        config.bitstream_is_big_endian = (flags1 & 64) != 0;
        config.simplified_prob_update = (flags1 & 128) != 0;
        config.no_repeated_offsets = (flags2 & 32) != 0;
        config.eof_in_length = (flags2 & 1) != 0;
        config.max_offset = if (flags2 & 2) == 0 { usize::MAX } else { 32 };
        config.max_length = if (flags2 & 4) == 0 { usize::MAX } else { 5 };
        level = (flags2 >> 3) & 3;
    }
    let packed = upkr::pack(data, level, &config, None);
    let unpacked = upkr::unpack(&packed, &config, 1024 * 1024).unwrap();
    assert!(unpacked == data);
});
