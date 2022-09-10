use crate::rans::{ONE_PROB, PROB_BITS};

const INIT_PROB: u16 = 1 << (PROB_BITS - 1);
const UPDATE_RATE: i32 = 4;
const UPDATE_ADD: i32 = 8;

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
        let offset = if !bit {
            ONE_PROB as i32 >> UPDATE_RATE
        } else {
            0
        };

        self.state.contexts[self.index] =
            (offset + old as i32 - ((old as i32 + UPDATE_ADD) >> UPDATE_RATE)) as u8;
    }
}
