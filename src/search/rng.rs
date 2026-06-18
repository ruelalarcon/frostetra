use rand::rngs::StdRng;
use rand::{thread_rng, Rng, SeedableRng};

pub enum SearchRng {
    Entropy,
    Seeded(parking_lot::Mutex<StdRng>),
}

impl SearchRng {
    pub fn from_seed(seed: u64) -> Self {
        SearchRng::Seeded(parking_lot::Mutex::new(StdRng::seed_from_u64(seed)))
    }

    pub fn from_entropy() -> Self {
        SearchRng::Entropy
    }

    pub fn gen_f64(&self) -> f64 {
        match self {
            SearchRng::Entropy => thread_rng().gen(),
            SearchRng::Seeded(rng) => rng.lock().gen(),
        }
    }

    pub fn gen_index(&self, upper: usize) -> usize {
        match self {
            SearchRng::Entropy => thread_rng().gen_range(0..upper),
            SearchRng::Seeded(rng) => rng.lock().gen_range(0..upper),
        }
    }
}
