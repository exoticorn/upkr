/// Heatmap information about a compressed block of data.
///
/// For each byte in the uncompressed data, the heatmap provides two pieces of intormation:
/// 1. whether this byte was encoded as a literal or as part of a match
/// 2. how many (fractional) bits where spend on encoding this byte
///
/// For the sake of the heatmap, the cost of literals are spread out across all matches
/// that reference the literal.
///
/// If the `terminal` feature is enabled, there is a function to write out the
/// heatmap as a colored hexdump.
pub struct Heatmap {
    data: Vec<u8>,
    cost: Vec<f32>,
    raw_cost: Vec<f32>,
    literal_index: Vec<usize>,
}

impl Heatmap {
    pub(crate) fn new() -> Heatmap {
        Heatmap {
            data: Vec::new(),
            cost: Vec::new(),
            raw_cost: Vec::new(),
            literal_index: Vec::new(),
        }
    }

    pub(crate) fn add_literal(&mut self, byte: u8, cost: f32) {
        self.data.push(byte);
        self.cost.push(cost);
        self.literal_index.push(self.literal_index.len());
    }

    pub(crate) fn add_match(&mut self, offset: usize, length: usize, mut cost: f32) {
        cost /= length as f32;
        for _ in 0..length {
            self.data.push(self.data[self.data.len() - offset]);
            self.literal_index
                .push(self.literal_index[self.literal_index.len() - offset]);
            self.cost.push(cost);
        }
    }

    pub(crate) fn finish(&mut self) {
        self.raw_cost = self.cost.clone();

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

    /// Reverses the heatmap
    pub fn reverse(&mut self) {
        self.data.reverse();
        self.cost.reverse();
        self.literal_index.reverse();
        for index in self.literal_index.iter_mut() {
            *index = self.data.len() - *index;
        }
    }

    /// The number of (uncompressed) bytes of data in this heatmap
    pub fn len(&self) -> usize {
        self.cost.len()
    }

    /// Returns whether the byte at `index` was encoded as a literal
    pub fn is_literal(&self, index: usize) -> bool {
        self.literal_index[index] == index
    }

    /// Returns the cost of encoding the byte at `index` in (fractional) bits.
    /// The cost of literal bytes is spread across the matches that reference it.
    /// See `raw_cost` for the raw encoding cost of each byte.
    pub fn cost(&self, index: usize) -> f32 {
        self.cost[index]
    }

    /// Returns the raw cost of encoding the byte at `index` in (fractional) bits
    pub fn raw_cost(&self, index: usize) -> f32 {
        self.raw_cost[index]
    }

    /// Returns the uncompressed data byte at `index`
    pub fn byte(&self, index: usize) -> u8 {
        self.data[index]
    }

    #[cfg(feature = "crossterm")]
    /// Print the heatmap as a colored hexdump
    pub fn print_as_hex(&self) -> std::io::Result<()> {
        self.print_as_hex_internal(false)
    }

    #[cfg(feature = "crossterm")]
    /// Print the heatmap as a colored hexdump, based on `raw_cost`.
    pub fn print_as_hex_raw_cost(&self) -> std::io::Result<()> {
        self.print_as_hex_internal(true)
    }

    #[cfg(feature = "crossterm")]
    fn print_as_hex_internal(&self, report_raw_cost: bool) -> std::io::Result<()> {
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
            report_raw_cost: bool,
        ) -> std::io::Result<()> {
            let cost = if report_raw_cost {
                heatmap.raw_cost(index)
            } else {
                heatmap.cost(index)
            };
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
                set_color(&mut stdout, self, i, num_colors, report_raw_cost)?;
                stdout.queue(Print(&format!("{:02x} ", self.data[i])))?;
            }

            let num_spaces = 1 + (bytes_per_row - (row_range.end - row_range.start)) * 3;
            let gap: String = std::iter::repeat(' ').take(num_spaces).collect();
            stdout
                .queue(SetAttribute(Attribute::Reset))?
                .queue(Print(&gap))?;

            for i in row_range.clone() {
                set_color(&mut stdout, self, i, num_colors, report_raw_cost)?;
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
