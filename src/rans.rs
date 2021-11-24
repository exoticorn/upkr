use crate::context_state::Context;

const L_BITS: u32 = 16;
pub const PROB_BITS: u32 = 12;
pub const ONE_PROB: u32 = 1 << PROB_BITS;

pub trait EntropyCoder {
    fn encode_bit(&mut self, bit: bool, prob: u16);

    fn encode_with_context(&mut self, bit: bool, context: &mut Context) {
        self.encode_bit(bit, context.prob());
        context.update(bit);
    }
}

pub struct RansCoder(Vec<u16>);

impl EntropyCoder for RansCoder {
    fn encode_bit(&mut self, bit: bool, prob: u16) {
        assert!(prob < 32768);
        self.0.push(prob | ((bit as u16) << 15));
    }
}

impl RansCoder {
    pub fn new() -> RansCoder {
        RansCoder(Vec::new())
    }

    pub fn finish(self) -> Vec<u8> {
        let mut buffer = vec![];
        let mut state = 1 << L_BITS;

        const MAX_STATE_FACTOR: u32 = 1 << (L_BITS + 8 - PROB_BITS);
        for step in self.0.into_iter().rev() {
            let prob = step as u32 & 32767;
            let (start, prob) = if step & 32768 != 0 {
                (0, prob)
            } else {
                (prob, ONE_PROB - prob)
            };
            let max_state = MAX_STATE_FACTOR * prob;
            while state >= max_state {
                buffer.push(state as u8);
                state >>= 8;
            }
            state = ((state / prob) << PROB_BITS) + (state % prob) + start;
        }

        while state > 0 {
            buffer.push(state as u8);
            state >>= 8;
        }

        buffer.reverse();
        buffer
    }
}

pub struct CostCounter(pub f64);

impl EntropyCoder for CostCounter {
    fn encode_bit(&mut self, bit: bool, prob: u16) {
        let prob = if bit { prob as u32 } else { ONE_PROB - prob as u32 };
        let inv_prob = ONE_PROB as f64 / prob as f64;
        self.0 += inv_prob.log2();
    }
}

pub struct RansDecoder<'a> {
    data: &'a [u8],
    state: u32,
}

const PROB_MASK: u32 = ONE_PROB - 1;
const L: u32 = 1 << L_BITS;

impl<'a> RansDecoder<'a> {
    pub fn new(data: &'a [u8]) -> RansDecoder<'a> {
        RansDecoder { data, state: 0 }
    }

    pub fn decode_with_context(&mut self, context: &mut Context) -> bool {
        let bit = self.decode_bit(context.prob());
        context.update(bit);
        bit
    }

    pub fn decode_bit(&mut self, prob: u16) -> bool {
        let prob = prob as u32;
        while self.state < L {
            self.state = (self.state << 8) | self.data[0] as u32;
            self.data = &self.data[1..];
        }

        let bit = (self.state & PROB_MASK) < prob;

        let (start, prob) = if bit {
            (0, prob)
        } else {
            (prob, ONE_PROB - prob)
        };
        self.state = prob * (self.state >> PROB_BITS) + (self.state & PROB_MASK) - start;

        bit
    }
}
