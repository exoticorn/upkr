pub struct Heatmap {
    data: Vec<u8>,
    cost: Vec<f32>,
    literal_index: Vec<usize>,
}

impl Heatmap {
    pub fn new() -> Heatmap {
        Heatmap {
            data: Vec::new(),
            cost: Vec::new(),
            literal_index: Vec::new(),
        }
    }

    pub fn add_literal(&mut self, byte: u8, cost: f32) {
        self.data.push(byte);
        self.cost.push(cost);
        self.literal_index.push(self.literal_index.len());
    }

    pub fn add_match(&mut self, offset: usize, length: usize, mut cost: f32) {
        cost /= length as f32;
        for _ in 0..length {
            self.data.push(self.data[self.data.len() - offset]);
            self.literal_index
                .push(self.literal_index[self.literal_index.len() - offset]);
            self.cost.push(cost);
        }
    }

    pub fn finish(&mut self) {
        let mut ref_count = vec![0usize; self.literal_index.len()];
        for &index in &self.literal_index {
            ref_count[index] += 1;
        }

        let mut shifted = vec![];
        for (&index, &cost) in self.literal_index.iter().zip(self.cost.iter()) {
            let delta = (self.cost[index] - cost) / ref_count[index] as f32;
            shifted.push(delta);
            shifted[index] -= delta;
        }

        for (cost, delta) in self.cost.iter_mut().zip(shifted.into_iter()) {
            *cost += delta;
        }
    }

    pub fn reverse(&mut self) {
        self.data.reverse();
        self.cost.reverse();
        self.literal_index.reverse();
        for index in self.literal_index.iter_mut() {
            *index = self.data.len() - *index;
        }
    }

    pub fn len(&self) -> usize {
        self.cost.len()
    }

    pub fn is_literal(&self, index: usize) -> bool {
        self.literal_index[index] == index
    }

    pub fn cost(&self, index: usize) -> f32 {
        self.cost[index]
    }

    pub fn byte(&self, index: usize) -> u8 {
        self.data[index]
    }
}
