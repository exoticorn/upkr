use std::ffi::c_int;

// the upkr config to use, this can be modified to use other configs
fn config() -> upkr::Config {
    upkr::Config::default()
}

#[no_mangle]
pub extern "C" fn upkr_compress(
    output_buffer: *mut u8,
    output_buffer_size: usize,
    input_buffer: *const u8,
    input_size: usize,
    compression_level: c_int,
) -> usize {
    let output_buffer = unsafe { std::slice::from_raw_parts_mut(output_buffer, output_buffer_size) };
    let input_buffer = unsafe { std::slice::from_raw_parts(input_buffer, input_size) };

    let packed_data = upkr::pack(input_buffer, compression_level.max(0).min(9) as u8, &config(), None);
    let copy_size = packed_data.len().min(output_buffer.len());
    output_buffer[..copy_size].copy_from_slice(&packed_data[..copy_size]);

    packed_data.len()
}

#[no_mangle]
pub extern "C" fn upkr_uncompress(output_buffer: *mut u8, output_buffer_size: usize, input_buffer: *const u8, input_size: usize) -> isize {
    let output_buffer = unsafe { std::slice::from_raw_parts_mut(output_buffer, output_buffer_size)};
    let input_buffer = unsafe { std::slice::from_raw_parts(input_buffer, input_size)};

    match upkr::unpack(input_buffer, &config(), output_buffer.len()) {
        Ok(unpacked_data) => {
            output_buffer[..unpacked_data.len()].copy_from_slice(&unpacked_data);
            unpacked_data.len() as isize
        }
        Err(upkr::UnpackError::OverSize { size, .. }) => size as isize,
        Err(other) => {
            eprintln!("[upkr] compressed data corrupt: {}", other);
            -1
        }
    }
}