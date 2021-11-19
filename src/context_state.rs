const INIT_PROB: u16 = 0x8000;
const UPDATE_RATE: u32 = 4;

#[derive(Clone)]
pub struct ContextState {
    contexts: Vec<u16>,
}

pub struct Context<'a> {
    state: &'a mut ContextState,
    index: usize,
}

impl ContextState {
    pub fn new(size: usize) -> ContextState {
        ContextState {
            contexts: vec![INIT_PROB; size],
        }
    }

    pub fn context_mut(&mut self, index: usize) -> Context {
        Context { state: self, index }
    }
}

impl<'a> Context<'a> {
    pub fn prob(&self) -> u16 {
        self.state.contexts[self.index]
    }

    pub fn update(&mut self, bit: bool) {
        let old = self.state.contexts[self.index];
        self.state.contexts[self.index] = if bit {
            old + (((1 << 16) - old as u32) >> UPDATE_RATE) as u16
        } else {
            old - (old >> UPDATE_RATE)
        };
    }
}
