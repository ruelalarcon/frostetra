use crate::search::SearchRng;

pub struct SearchContext {
    pub rng: SearchRng,
}

impl SearchContext {
    pub fn from_seed(seed: u64) -> Self {
        SearchContext {
            rng: SearchRng::from_seed(seed),
        }
    }

    pub fn from_entropy() -> Self {
        SearchContext {
            rng: SearchRng::from_entropy(),
        }
    }
}
