use crate::rans::{PROB_BITS, ONE_PROB};

const INIT_PROB: u16 = 1 << (PROB_BITS - 1);
const UPDATE_RATE: u32 = 4;
const UPDATE_ADD: u32 = 8;

#[derive(Clone)]
pub struct ContextState {
    contexts: Vec<u8>,
}

pub struct Context<'a> {
    state: &'a mut ContextState,
    index: usize,
}

impl ContextState {
    pub fn new(size: usize) -> ContextState {
        ContextState {
            contexts: vec![INIT_PROB as u8; size],
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
        self.state.contexts[self.index] = if bit {
            old + ((ONE_PROB - old as u32 + UPDATE_ADD) >> UPDATE_RATE) as u8
        } else {
            old - ((old as u32 + UPDATE_ADD) >> UPDATE_RATE) as u8
        };
    }
}
