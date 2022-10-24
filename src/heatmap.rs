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

    pub fn print_as_hex(&self) -> std::io::Result<()> {
        use crossterm::{
            style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor},
            QueueableCommand,
        };
        use std::io::{stdout, Write};

        fn set_color(
            mut out: impl QueueableCommand,
            heatmap: &Heatmap,
            index: usize,
            num_colors: u16,
        ) -> std::io::Result<()> {
            let cost = heatmap.cost(index);
            if num_colors < 256 {
                let colors = [
                    Color::Red,
                    Color::Yellow,
                    Color::Green,
                    Color::Cyan,
                    Color::Blue,
                    Color::DarkBlue,
                    Color::Black,
                ];
                let color_index = (3. - cost.log2())
                    .round()
                    .max(0.)
                    .min((colors.len() - 1) as f32) as usize;
                out.queue(SetBackgroundColor(colors[color_index]))?;
            } else {
                let colors = [
                    196, 166, 136, 106, 76, 46, 41, 36, 31, 26, 21, 20, 19, 18, 17, 16,
                ];
                let color_index = ((3. - cost.log2()) * 2.5)
                    .round()
                    .max(0.)
                    .min((colors.len() - 1) as f32) as usize;
                out.queue(SetBackgroundColor(Color::AnsiValue(colors[color_index])))?;
            }
            out.queue(SetAttribute(if heatmap.is_literal(index) {
                Attribute::Underlined
            } else {
                Attribute::NoUnderline
            }))?;
            Ok(())
        }

        let num_colors = crossterm::style::available_color_count();

        let term_width = crossterm::terminal::size()?.0.min(120) as usize;
        let bytes_per_row = (term_width - 8) / 4;

        for row_start in (0..self.data.len()).step_by(bytes_per_row) {
            let row_range = row_start..self.data.len().min(row_start + bytes_per_row);
            let mut stdout = stdout();

            stdout.queue(Print(&format!("{:04x}  ", row_start)))?;

            for i in row_range.clone() {
                set_color(&mut stdout, self, i, num_colors)?;
                stdout.queue(Print(&format!("{:02x} ", self.data[i])))?;
            }

            let num_spaces = 1 + (bytes_per_row - (row_range.end - row_range.start)) * 3;
            let gap: String = std::iter::repeat(' ').take(num_spaces).collect();
            stdout
                .queue(SetAttribute(Attribute::Reset))?
                .queue(Print(&gap))?;

            for i in row_range.clone() {
                set_color(&mut stdout, self, i, num_colors)?;
                let byte = self.data[i];
                if byte >= 32 && byte < 127 {
                    stdout.queue(Print(format!("{}", byte as char)))?;
                } else {
                    stdout.queue(Print("."))?;
                }
            }

            stdout
                .queue(SetAttribute(Attribute::Reset))?
                .queue(Print("\n"))?;

            stdout.flush()?;
        }

        Ok(())
    }
}
