use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;

use nohash::IntMap;
use parking_lot::MappedRwLockReadGuard;
use parking_lot::MappedRwLockWriteGuard;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;

use crate::tetris::model::GameState;

pub struct StateMap<V, S = StableStateHasher> {
    hasher: S,
    buckets: Box<[RwLock<IntMap<u64, V>>]>,
}

#[derive(Clone)]
pub struct StableStateHasher(ahash::RandomState);

impl Default for StableStateHasher {
    fn default() -> Self {
        StableStateHasher(ahash::RandomState::with_seeds(
            0x243f_6a88_85a3_08d3,
            0x1319_8a2e_0370_7344,
            0xa409_3822_299f_31d0,
            0x082e_fa98_ec4e_6c89,
        ))
    }
}

impl BuildHasher for StableStateHasher {
    type Hasher = <ahash::RandomState as BuildHasher>::Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        self.0.build_hasher()
    }
}

const SHARD_INDEX_BITS: usize = 12;
const SHARD_INDEX_SHIFT: usize = 32;
const SHARDS: usize = 1 << SHARD_INDEX_BITS;

impl<V, S: Default> Default for StateMap<V, S> {
    fn default() -> Self {
        StateMap {
            hasher: Default::default(),
            buckets: std::iter::repeat_with(|| RwLock::new(IntMap::default()))
                .take(SHARDS)
                .collect(),
        }
    }
}

impl<V, S: BuildHasher> StateMap<V, S> {
    pub fn index(&self, k: &GameState) -> u64 {
        let mut hasher = self.hasher.build_hasher();
        k.hash(&mut hasher);
        hasher.finish()
    }

    fn bucket(&self, k: u64) -> &RwLock<IntMap<u64, V>> {
        &self.buckets[(k >> SHARD_INDEX_SHIFT) as usize % SHARDS]
    }

    pub fn get_raw(&self, k: u64) -> Option<MappedRwLockReadGuard<'_, V>> {
        RwLockReadGuard::try_map(self.bucket(k).read(), |shard| shard.get(&k)).ok()
    }

    pub fn get(&self, k: &GameState) -> Option<MappedRwLockReadGuard<'_, V>> {
        self.get_raw(self.index(k))
    }

    pub fn get_raw_mut(&self, k: u64) -> Option<MappedRwLockWriteGuard<'_, V>> {
        RwLockWriteGuard::try_map(self.bucket(k).write(), |shard| shard.get_mut(&k)).ok()
    }

    pub fn get_raw_or_insert_with(
        &self,
        k: u64,
        f: impl FnOnce() -> V,
    ) -> MappedRwLockWriteGuard<'_, V> {
        RwLockWriteGuard::map(self.bucket(k).write(), |shard| {
            shard.entry(k).or_insert_with(f)
        })
    }

    pub fn get_or_insert_with(
        &self,
        k: &GameState,
        f: impl FnOnce() -> V,
    ) -> MappedRwLockWriteGuard<'_, V> {
        self.get_raw_or_insert_with(self.index(k), f)
    }

    pub fn map_values<T>(self, f: impl Fn(V) -> T) -> StateMap<T, S> {
        StateMap {
            hasher: self.hasher,
            buckets: self
                .buckets
                .into_vec()
                .into_iter()
                .map(|shard| {
                    RwLock::new(
                        shard
                            .into_inner()
                            .into_iter()
                            .map(|(k, v)| (k, f(v)))
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}
