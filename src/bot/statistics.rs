#[derive(Copy, Clone, Debug, Default)]
pub struct Statistics {
    pub nodes: u64,
    pub selections: u64,
    pub expansions: u64,
}

impl Statistics {
    pub fn accumulate(&mut self, other: Self) {
        self.nodes += other.nodes;
        self.selections += other.selections;
        self.expansions += other.expansions;
    }
}
