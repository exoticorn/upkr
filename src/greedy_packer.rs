use crate::lz;
use crate::match_finder::MatchFinder;
use crate::rans::RansCoder;
use crate::ProgressCallback;

pub fn pack(
    data: &[u8],
    use_bitstream: bool,
    parity_contexts: usize,
    mut progress_callback: Option<ProgressCallback>,
) -> Vec<u8> {
    let mut match_finder = MatchFinder::new(data);
    let mut rans_coder = RansCoder::new(use_bitstream);
    let mut state = lz::CoderState::new(parity_contexts);

    let mut pos = 0;
    while pos < data.len() {
        if let Some(ref mut cb) = progress_callback {
            cb(pos);
        }
        let mut encoded_match = false;
        if let Some(m) = match_finder.matches(pos).next() {
            let max_offset = 1 << (m.length * 3 - 1).min(31);
            let offset = pos - m.pos;
            if offset < max_offset {
                lz::Op::Match {
                    offset: offset as u32,
                    len: m.length as u32,
                }
                .encode(&mut rans_coder, &mut state);
                pos += m.length;
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
                    .count();
                if length > 0 {
                    lz::Op::Match {
                        offset: offset as u32,
                        len: length as u32,
                    }
                    .encode(&mut rans_coder, &mut state);
                    pos += length;
                    encoded_match = true;
                }
            }
        }

        if !encoded_match {
            lz::Op::Literal(data[pos]).encode(&mut rans_coder, &mut state);
            pos += 1;
        }
    }

    lz::encode_eof(&mut rans_coder, &mut state);
    rans_coder.finish()
}
