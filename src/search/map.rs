use std::cell::UnsafeCell;
use std::hash::BuildHasher;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use nohash::IntMap;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;

use crate::tetris::model::GameState;

pub struct StateMap<V, S = StableStateHasher> {
    hasher: S,
    locking: bool,
    buckets: Box<[Bucket<V>]>,
}

struct Bucket<V> {
    lock: RwLock<()>,
    map: UnsafeCell<IntMap<u64, V>>,
}

#[derive(Clone)]
pub struct StableStateHasher(ahash::RandomState);

pub struct StateReadGuard<'a, V> {
    _lock: Option<RwLockReadGuard<'a, ()>>,
    value: *const V,
    _marker: PhantomData<&'a V>,
}

pub struct StateWriteGuard<'a, V> {
    _lock: Option<RwLockWriteGuard<'a, ()>>,
    value: *mut V,
    _marker: PhantomData<&'a mut V>,
}

// Locked maps protect bucket access with `lock`. Local maps may only be used
// behind BotRunner's thread-owner guard, which is checked before search enters
// the DAG.
unsafe impl<V: Send> Sync for Bucket<V> {}
unsafe impl<V: Send> Send for Bucket<V> {}

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
        Self::new(true)
    }
}

impl<V, S: Default> StateMap<V, S> {
    pub(crate) fn new(locking: bool) -> Self {
        let shards = if locking { SHARDS } else { 1 };
        StateMap {
            hasher: Default::default(),
            locking,
            buckets: std::iter::repeat_with(|| Bucket {
                lock: RwLock::new(()),
                map: UnsafeCell::new(IntMap::default()),
            })
            .take(shards)
            .collect(),
        }
    }
}

impl<V, S: BuildHasher> StateMap<V, S> {
    pub fn index(&self, k: &GameState) -> u64 {
        let mut hasher = self.hasher.build_hasher();
        for &col in &k.board.cols {
            hasher.write_u64(col);
        }

        let metadata = k.bag.as_u64()
            | ((k.reserve as u64) << 8)
            | ((k.back_to_back as u64) << 16)
            | ((k.combo as u64) << 24);
        hasher.write_u64(metadata);
        hasher.finish()
    }

    fn bucket(&self, k: u64) -> &Bucket<V> {
        &self.buckets[(k >> SHARD_INDEX_SHIFT) as usize % self.buckets.len()]
    }

    pub fn get_raw(&self, k: u64) -> Option<StateReadGuard<'_, V>> {
        let bucket = self.bucket(k);
        let lock = self.locking.then(|| bucket.lock.read());
        // SAFETY: locked maps hold the bucket read lock; local maps have
        // already passed BotRunner's thread-owner guard before search reached
        // the DAG.
        let value = unsafe { (&*bucket.map.get()).get(&k)? as *const V };
        Some(StateReadGuard {
            _lock: lock,
            value,
            _marker: PhantomData,
        })
    }

    pub fn get(&self, k: &GameState) -> Option<StateReadGuard<'_, V>> {
        self.get_raw(self.index(k))
    }

    pub fn get_raw_mut(&self, k: u64) -> Option<StateWriteGuard<'_, V>> {
        let bucket = self.bucket(k);
        let lock = self.locking.then(|| bucket.lock.write());
        // SAFETY: locked maps hold the bucket write lock; local maps have
        // already passed BotRunner's thread-owner guard before search reached
        // the DAG.
        let value = unsafe { (&mut *bucket.map.get()).get_mut(&k)? as *mut V };
        Some(StateWriteGuard {
            _lock: lock,
            value,
            _marker: PhantomData,
        })
    }

    pub fn get_raw_or_insert_with(&self, k: u64, f: impl FnOnce() -> V) -> StateWriteGuard<'_, V> {
        let bucket = self.bucket(k);
        let lock = self.locking.then(|| bucket.lock.write());
        // SAFETY: locked maps hold the bucket write lock; local maps have
        // already passed BotRunner's thread-owner guard before search reached
        // the DAG.
        let value = unsafe { (&mut *bucket.map.get()).entry(k).or_insert_with(f) as *mut V };
        StateWriteGuard {
            _lock: lock,
            value,
            _marker: PhantomData,
        }
    }

    pub fn get_or_insert_with(
        &self,
        k: &GameState,
        f: impl FnOnce() -> V,
    ) -> StateWriteGuard<'_, V> {
        self.get_raw_or_insert_with(self.index(k), f)
    }

    pub fn map_values<T>(self, f: impl Fn(V) -> T) -> StateMap<T, S> {
        StateMap {
            hasher: self.hasher,
            locking: self.locking,
            buckets: self
                .buckets
                .into_vec()
                .into_iter()
                .map(|shard| {
                    let map = shard
                        .map
                        .into_inner()
                        .into_iter()
                        .map(|(k, v)| (k, f(v)))
                        .collect();
                    Bucket {
                        lock: RwLock::new(()),
                        map: UnsafeCell::new(map),
                    }
                })
                .collect(),
        }
    }
}

impl<V> Deref for StateReadGuard<'_, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<V> Deref for StateWriteGuard<'_, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.value }
    }
}

impl<V> DerefMut for StateWriteGuard<'_, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.value }
    }
}
