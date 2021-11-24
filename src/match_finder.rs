use std::collections::BinaryHeap;
use std::ops::Range;

pub struct MatchFinder {
    suffixes: Vec<i32>,
    rev_suffixes: Vec<u32>,
    lcp: Vec<u32>,

    max_queue_size: usize,
    max_matches_per_length: usize,
    patience: usize,
    max_length_diff: usize,
}

impl MatchFinder {
    pub fn new(data: &[u8]) -> MatchFinder {
        let mut suffixes = vec![0i32; data.len()];
        cdivsufsort::sort_in_place(data, &mut suffixes);

        let mut rev_suffixes = vec![0u32; data.len()];
        for (suffix_index, index) in suffixes.iter().enumerate() {
            rev_suffixes[*index as usize] = suffix_index as u32;
        }

        let mut lcp = vec![0u32; data.len()];
        let mut length = 0usize;
        for suffix_index in &rev_suffixes {
            if *suffix_index as usize + 1 < suffixes.len() {
                let i = suffixes[*suffix_index as usize] as usize;
                let j = suffixes[*suffix_index as usize + 1] as usize;
                while i + length < data.len()
                    && j + length < data.len()
                    && data[i + length] == data[j + length]
                {
                    length += 1;
                }
                lcp[*suffix_index as usize] = length as u32;
            }
            length = length.saturating_sub(1);
        }

        MatchFinder {
            suffixes,
            rev_suffixes,
            lcp,
            max_queue_size: 100,
            max_matches_per_length: 5,
            patience: 100,
            max_length_diff: 2,
        }
    }

    pub fn matches(&self, pos: usize) -> Matches {
        let index = self.rev_suffixes[pos] as usize;
        let mut matches = Matches {
            finder: self,
            pos_range: 0..pos,
            left_index: index,
            left_length: usize::MAX,
            right_index: index,
            right_length: usize::MAX,
            current_length: usize::MAX,
            matches_left: 0,
            max_length: 0,
            queue: BinaryHeap::new(),
        };

        matches.move_left();
        matches.move_right();

        matches
    }
}

pub struct Matches<'a> {
    finder: &'a MatchFinder,
    pos_range: Range<usize>,
    left_index: usize,
    left_length: usize,
    right_index: usize,
    right_length: usize,
    current_length: usize,
    matches_left: usize,
    max_length: usize,
    queue: BinaryHeap<usize>,
}

#[derive(Debug)]
pub struct Match {
    pub pos: usize,
    pub length: usize,
}

impl<'a> Iterator for Matches<'a> {
    type Item = Match;

    fn next(&mut self) -> Option<Match> {
        if self.queue.is_empty() || self.matches_left == 0 {
            self.queue.clear();
            self.current_length = self.current_length.saturating_sub(1).min(self.left_length.max(self.right_length));
            self.max_length = self.max_length.max(self.current_length);
            if self.current_length < 2
                || self.current_length + self.finder.max_length_diff < self.max_length
            {
                return None;
            }
            while self.queue.len() < self.finder.max_queue_size
                && (self.left_length == self.current_length
                    || self.right_length == self.current_length)
            {
                if self.left_length == self.current_length {
                    self.add_to_queue(self.finder.suffixes[self.left_index]);
                    self.move_left();
                }
                if self.right_length == self.current_length {
                    self.add_to_queue(self.finder.suffixes[self.right_index]);
                    self.move_right();
                }
            }
            self.matches_left = self.finder.max_matches_per_length;
        }

        self.matches_left = self.matches_left.saturating_sub(1);

        self.queue.pop().map(|pos| Match {
            pos,
            length: self.current_length,
        })
    }
}

impl<'a> Matches<'a> {
    fn move_left(&mut self) {
        let mut patience = self.finder.patience;
        while self.left_length > 0 && patience > 0 && self.left_index > 0 {
            self.left_index -= 1;
            self.left_length = self
                .left_length
                .min(self.finder.lcp[self.left_index] as usize);
            if self
                .pos_range
                .contains(&(self.finder.suffixes[self.left_index] as usize))
            {
                return;
            }
            patience -= 1;
        }
        self.left_length = 0;
    }

    fn move_right(&mut self) {
        let mut patience = self.finder.patience;
        while self.right_length > 0
            && patience > 0
            && self.right_index + 1 < self.finder.suffixes.len()
        {
            self.right_index += 1;
            self.right_length = self
                .right_length
                .min(self.finder.lcp[self.right_index - 1] as usize);
            if self
                .pos_range
                .contains(&(self.finder.suffixes[self.right_index] as usize))
            {
                return;
            }
            patience -= 1;
        }
        self.right_length = 0;
    }

    fn add_to_queue(&mut self, pos: i32) {
        self.queue.push(pos as usize);
    }
}
