use crate::search::SearchRng;

pub struct SearchContext {
    rng: SearchRng,
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

    pub fn gen_f64(&self) -> f64 {
        self.rng.gen_f64()
    }

    pub fn gen_index(&self, upper: usize) -> usize {
        self.rng.gen_index(upper)
    }
}
