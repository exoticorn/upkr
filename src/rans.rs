use crate::{context_state::Context, Config};
use thiserror::Error;

pub const PROB_BITS: u32 = 8;
pub const ONE_PROB: u32 = 1 << PROB_BITS;

pub trait EntropyCoder {
    fn encode_bit(&mut self, bit: bool, prob: u16);

    fn encode_with_context(&mut self, bit: bool, context: &mut Context) {
        self.encode_bit(bit, context.prob());
        context.update(bit);
    }
}

pub struct RansCoder {
    bits: Vec<u16>,
    use_bitstream: bool,
    bitstream_is_big_endian: bool,
    invert_bit_encoding: bool,
}

impl EntropyCoder for RansCoder {
    fn encode_bit(&mut self, bit: bool, prob: u16) {
        assert!(prob < 32768);
        self.bits
            .push(prob | (((bit ^ self.invert_bit_encoding) as u16) << 15));
    }
}

impl RansCoder {
    pub fn new(config: &Config) -> RansCoder {
        RansCoder {
            bits: Vec::new(),
            use_bitstream: config.use_bitstream,
            bitstream_is_big_endian: config.bitstream_is_big_endian,
            invert_bit_encoding: config.invert_bit_encoding,
        }
    }

    pub fn finish(self) -> Vec<u8> {
        let mut buffer = vec![];
        let l_bits: u32 = if self.use_bitstream { 15 } else { 12 };
        let mut state = 1 << l_bits;

        let mut byte = 0u8;
        let mut bit = if self.bitstream_is_big_endian { 0 } else { 8 };
        let mut flush_state: Box<dyn FnMut(&mut u32)> = if self.use_bitstream {
            if self.bitstream_is_big_endian {
                Box::new(|state: &mut u32| {
                    byte |= ((*state & 1) as u8) << bit;
                    bit += 1;
                    if bit == 8 {
                        buffer.push(byte);
                        byte = 0;
                        bit = 0;
                    }
                    *state >>= 1;
                })
            } else {
                Box::new(|state: &mut u32| {
                    bit -= 1;
                    byte |= ((*state & 1) as u8) << bit;
                    if bit == 0 {
                        buffer.push(byte);
                        byte = 0;
                        bit = 8;
                    }
                    *state >>= 1;
                })
            }
        } else {
            Box::new(|state: &mut u32| {
                buffer.push(*state as u8);
                *state >>= 8;
            })
        };

        let num_flush_bits = if self.use_bitstream { 1 } else { 8 };
        let max_state_factor: u32 = 1 << (l_bits + num_flush_bits - PROB_BITS);
        for step in self.bits.into_iter().rev() {
            let prob = step as u32 & 32767;
            let (start, prob) = if step & 32768 != 0 {
                (0, prob)
            } else {
                (prob, ONE_PROB - prob)
            };
            let max_state = max_state_factor * prob;
            while state >= max_state {
                flush_state(&mut state);
            }
            state = ((state / prob) << PROB_BITS) + (state % prob) + start;
        }

        while state > 0 {
            flush_state(&mut state);
        }

        drop(flush_state);

        if self.use_bitstream && byte != 0 {
            buffer.push(byte);
        }

        buffer.reverse();
        buffer
    }
}

pub struct CostCounter {
    cost: f64,
    log2_table: Vec<f64>,
    invert_bit_encoding: bool,
}

impl CostCounter {
    pub fn new(config: &Config) -> CostCounter {
        let log2_table = (0..ONE_PROB)
            .map(|prob| {
                let inv_prob = ONE_PROB as f64 / prob as f64;
                inv_prob.log2()
            })
            .collect();
        CostCounter {
            cost: 0.0,
            log2_table,
            invert_bit_encoding: config.invert_bit_encoding,
        }
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn reset(&mut self) {
        self.cost = 0.0;
    }
}

impl EntropyCoder for CostCounter {
    fn encode_bit(&mut self, bit: bool, prob: u16) {
        let prob = if bit ^ self.invert_bit_encoding {
            prob as u32
        } else {
            ONE_PROB - prob as u32
        };
        self.cost += self.log2_table[prob as usize];
    }
}

#[derive(Clone)]
pub struct RansDecoder<'a> {
    data: &'a [u8],
    pos: usize,
    state: u32,
    use_bitstream: bool,
    byte: u8,
    bits_left: u8,
    invert_bit_encoding: bool,
    bitstream_is_big_endian: bool,
}

const PROB_MASK: u32 = ONE_PROB - 1;

#[derive(Debug, Error)]
#[error("Unexpected end of input")]
pub struct UnexpectedEOF;

impl<'a> RansDecoder<'a> {
    pub fn new(data: &'a [u8], config: &Config) -> Result<RansDecoder<'a>, UnexpectedEOF> {
        let mut decoder = RansDecoder {
            data,
            pos: 0,
            state: 0,
            use_bitstream: config.use_bitstream,
            byte: 0,
            bits_left: 0,
            invert_bit_encoding: config.invert_bit_encoding,
            bitstream_is_big_endian: config.bitstream_is_big_endian,
        };
        decoder.refill()?;
        Ok(decoder)
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn decode_with_context(&mut self, context: &mut Context) -> Result<bool, UnexpectedEOF> {
        let bit = self.decode_bit(context.prob())?;
        context.update(bit);
        Ok(bit)
    }

    fn refill(&mut self) -> Result<(), UnexpectedEOF> {
        if self.use_bitstream {
            while self.state < 32768 {
                if self.bits_left == 0 {
                    if self.pos >= self.data.len() {
                        return Err(UnexpectedEOF);
                    }
                    self.byte = self.data[self.pos];
                    self.pos += 1;
                    self.bits_left = 8;
                }
                if self.bitstream_is_big_endian {
                    self.state = (self.state << 1) | (self.byte >> 7) as u32;
                    self.byte <<= 1;
                } else {
                    self.state = (self.state << 1) | (self.byte & 1) as u32;
                    self.byte >>= 1;
                }
                self.bits_left -= 1;
            }
        } else {
            while self.state < 4096 {
                if self.pos >= self.data.len() {
                    return Err(UnexpectedEOF);
                }
                self.state = (self.state << 8) | self.data[self.pos] as u32;
                self.pos += 1;
            }
        }
        Ok(())
    }

    pub fn decode_bit(&mut self, prob: u16) -> Result<bool, UnexpectedEOF> {
        self.refill()?;

        let prob = prob as u32;

        let bit = (self.state & PROB_MASK) < prob;

        let (start, prob) = if bit {
            (0, prob)
        } else {
            (prob, ONE_PROB - prob)
        };
        self.state = prob * (self.state >> PROB_BITS) + (self.state & PROB_MASK) - start;

        Ok(bit ^ self.invert_bit_encoding)
    }

    pub fn cost(&self, prev: &RansDecoder) -> f32 {
        f32::log2(prev.state as f32) - f32::log2(self.state as f32)
            + (self.pos - prev.pos) as f32 * 8.
    }
}
