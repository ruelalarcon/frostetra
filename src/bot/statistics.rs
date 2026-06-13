#[derive(Copy, Clone, Debug, Default)]
pub struct Statistics {
    pub nodes: u64,
    pub selections: u64,
    pub expansions: u64,
    pub max_depth: usize,
}

impl Statistics {
    pub fn accumulate(&mut self, other: Self) {
        self.nodes += other.nodes;
        self.selections += other.selections;
        self.expansions += other.expansions;
        self.max_depth = self.max_depth.max(other.max_depth);
    }
}
