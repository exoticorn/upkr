use crate::{
    rans::{ONE_PROB, PROB_BITS},
    Config,
};

const INIT_PROB: u16 = 1 << (PROB_BITS - 1);
const UPDATE_RATE: u32 = 4;
const UPDATE_ADD: u32 = 8;

#[derive(Clone)]
pub struct ContextState {
    contexts: Vec<u8>,
    invert_bit_encoding: bool,
    simplified_prob_update: bool,
}

pub struct Context<'a> {
    state: &'a mut ContextState,
    index: usize,
}

impl ContextState {
    pub fn new(size: usize, config: &Config) -> ContextState {
        ContextState {
            contexts: vec![INIT_PROB as u8; size],
            invert_bit_encoding: config.invert_bit_encoding,
            simplified_prob_update: config.simplified_prob_update,
        }
    }

    pub fn context_mut(&mut self, index: usize) -> Context {
        Context { state: self, index }
    }
}

impl<'a> Context<'a> {
    pub fn prob(&self) -> u16 {
        self.state.contexts[self.index] as u16
    }

    pub fn update(&mut self, bit: bool) {
        let old = self.state.contexts[self.index];

        self.state.contexts[self.index] = if self.state.simplified_prob_update {
            let offset = if bit ^ self.state.invert_bit_encoding {
                ONE_PROB as i32 >> UPDATE_RATE
            } else {
                0
            };

            (offset + old as i32 - ((old as i32 + UPDATE_ADD as i32) >> UPDATE_RATE)) as u8
        } else {
            if bit ^ self.state.invert_bit_encoding {
                old + ((ONE_PROB - old as u32 + UPDATE_ADD) >> UPDATE_RATE) as u8
            } else {
                old - ((old as u32 + UPDATE_ADD) >> UPDATE_RATE) as u8
            }
        };
    }
}
