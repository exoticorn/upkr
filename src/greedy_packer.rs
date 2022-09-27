use crate::match_finder::MatchFinder;
use crate::rans::RansCoder;
use crate::ProgressCallback;
use crate::{lz, Config};

pub fn pack(
    data: &[u8],
    config: &Config,
    mut progress_callback: Option<ProgressCallback>,
) -> Vec<u8> {
    let mut match_finder = MatchFinder::new(data);
    let mut rans_coder = RansCoder::new(config);
    let mut state = lz::CoderState::new(config);

    let mut pos = 0;
    while pos < data.len() {
        if let Some(ref mut cb) = progress_callback {
            cb(pos);
        }
        let mut encoded_match = false;
        if let Some(m) = match_finder.matches(pos).next() {
            let max_offset = config.max_offset.min(1 << (m.length * 3 - 1).min(31));
            let offset = pos - m.pos;
            if offset < max_offset && m.length >= config.min_length() {
                let length = m.length.min(config.max_length);
                lz::Op::Match {
                    offset: offset as u32,
                    len: length as u32,
                }
                .encode(&mut rans_coder, &mut state, config);
                pos += length;
                encoded_match = true;
            }
        }

        if !encoded_match {
            let offset = state.last_offset() as usize;
            if offset != 0 {
                let length = data[pos..]
                    .iter()
                    .zip(data[(pos - offset)..].iter())
                    .take_while(|(a, b)| a == b)
                    .count()
                    .min(config.max_length);
                if length >= config.min_length() {
                    lz::Op::Match {
                        offset: offset as u32,
                        len: length as u32,
                    }
                    .encode(&mut rans_coder, &mut state, config);
                    pos += length;
                    encoded_match = true;
                }
            }
        }

        if !encoded_match {
            lz::Op::Literal(data[pos]).encode(&mut rans_coder, &mut state, config);
            pos += 1;
        }
    }

    lz::encode_eof(&mut rans_coder, &mut state, config);
    rans_coder.finish()
}
