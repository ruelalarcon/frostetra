#[derive(Clone, Copy, Debug)]
pub enum SearchBudget {
    Iterations(u64),
    Nodes(u64),
}
