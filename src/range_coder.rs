use crate::context_state::Context;

pub struct RangeCoder {
    buffer: Vec<u8>,
    low: u64,
    range: u64,
}

const TOTAL: u32 = 65536;

impl RangeCoder {
    pub fn new() -> RangeCoder {
        RangeCoder {
            buffer: vec![],
            low: 0,
            range: 1 << 40,
        }
    }

    pub fn encode_with_context(&mut self, bit: bool, context: &mut Context) {
        self.encode_bit(bit, context.prob() as u32);
        context.update(bit);
    }

    pub fn encode_bit(&mut self, bit: bool, prob: u32) {
        let (start, size) = if bit { (0, prob) } else { (prob, TOTAL - prob) };
        self.range /= TOTAL as u64;
        self.low += start as u64 * self.range;
        self.range *= size as u64;

        while (self.low >> 32) == (self.low + self.range - 1) >> 32 {
            self.emit_byte();
        }

        if self.range < 1 << 24 {
            self.emit_byte();
            self.emit_byte();
            self.range = (1 << 40) - self.low;
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        while self.range < 1 << 32 {
            self.emit_byte();
        }
        self.low += 1 << 32;
        self.emit_byte();
        self.buffer
    }

    fn emit_byte(&mut self) {
        self.buffer.push((self.low >> 32).try_into().unwrap());
        self.low = (self.low & 0xffffffff) << 8;
        self.range *= 256;
    }
}

pub struct RangeDecoder<'a> {
    data: &'a [u8],
    code: u64,
    low: u64,
    range: u64,
}

impl<'a> RangeDecoder<'a> {
    pub fn new(data: &'a [u8]) -> RangeDecoder<'a> {
        RangeDecoder {
            data,
            code: 0,
            low: 0,
            range: 1,
        }
    }

    pub fn decode_with_context(&mut self, context: &mut Context) -> bool {
        let bit = self.decode_bit(context.prob() as u32);
        context.update(bit);
        bit
    }

    pub fn decode_bit(&mut self, prob: u32) -> bool {
        while self.low >> 32 == (self.low + self.range - 1) >> 32 {
            self.append_byte();
        }

        if self.range < 1 << 24 {
            self.append_byte();
            self.append_byte();
            self.range = (1 << 40) - self.low;
        }

        let bit = (self.code - self.low) / (self.range / TOTAL as u64) < prob as u64;

        let (start, size) = if bit { (0, prob) } else { (prob, TOTAL - prob) };
        self.range /= TOTAL as u64;
        self.low += start as u64 * self.range;
        self.range *= size as u64;

        bit
    }

    fn append_byte(&mut self) {
        self.code = (self.code & 0xffffffff) << 8;
        if !self.data.is_empty() {
            self.code |= self.data[0] as u64;
            self.data = &self.data[1..];
        }
        self.low = (self.low & 0xffffffff) << 8;
        self.range <<= 8;
    }
}
