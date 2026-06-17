use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct SearchRng {
    inner: StdRng,
}

impl SearchRng {
    pub fn from_seed(seed: u64) -> Self {
        SearchRng {
            inner: StdRng::seed_from_u64(seed),
        }
    }

    pub fn from_entropy() -> Self {
        SearchRng {
            inner: StdRng::from_entropy(),
        }
    }

    pub fn gen_f64(&mut self) -> f64 {
        self.inner.gen()
    }

    pub fn gen_index(&mut self, upper: usize) -> usize {
        self.inner.gen_range(0..upper)
    }
}
