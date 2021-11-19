use crate::lz::LzCoder;
use crate::match_finder::MatchFinder;

pub fn pack(data: &[u8]) -> Vec<u8> {
    let match_finder = MatchFinder::new(data);
    let mut lz = LzCoder::new();

    let mut pos = 0;
    while pos < data.len() {
        let mut encoded_match = false;
        if let Some(m) = match_finder.matches(pos).next() {
            let max_offset = 1 << (m.length * 3 - 1).min(31);
            let offset = pos - m.pos;
            if offset < max_offset {
                lz.encode_match(offset, m.length);
                pos += m.length;
                encoded_match = true;
            }
        }

        if !encoded_match {
            let offset = lz.last_offset();
            if offset != 0 {
                let length = data[pos..]
                    .iter()
                    .zip(data[(pos - offset)..].iter())
                    .take_while(|(a, b)| a == b)
                    .count();
                if length > 0 {
                    lz.encode_match(offset, length);
                    pos += length;
                    encoded_match = true;
                }
            }
        }

        if !encoded_match {
            lz.encode_literal(data[pos]);
            pos += 1;
        }
    }

    lz.finish()
}
