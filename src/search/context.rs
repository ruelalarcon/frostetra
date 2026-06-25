use crate::config::SearchRngConfig;
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

    /// Creates an independent random stream for one background worker.
    ///
    /// A shared seeded RNG turns every selection into a mutex bottleneck.  A
    /// worker-local stream keeps each worker distinct without serializing
    /// workers on random-number draws.
    pub fn from_rng_config(config: &SearchRngConfig, worker: usize) -> Self {
        match config {
            SearchRngConfig::Entropy => Self::from_entropy(),
            SearchRngConfig::Seeded { seed } => Self::from_seed(split_seed(*seed, worker as u64)),
        }
    }

    pub fn gen_f64(&self) -> f64 {
        self.rng.gen_f64()
    }

    pub fn gen_index(&self, upper: usize) -> usize {
        self.rng.gen_index(upper)
    }
}

fn split_seed(seed: u64, stream: u64) -> u64 {
    let mut value = seed.wrapping_add(stream.wrapping_add(1).wrapping_mul(0x9e37_79b9_7f4a_7c15));
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}
