use std::{array, marker::PhantomData};

pub trait CacheHash {
    fn hash(&self) -> u64;
}

#[derive(Clone)]
struct Entry<T> {
    value: T,
    hash: u64,
}

impl<T: Copy> Copy for Entry<T> {}

#[derive(Clone)]
pub struct Cache<K: CacheHash, V, const N: usize> {
    data: [Option<Entry<V>>; N],
    _marker: PhantomData<K>,
}

impl<K: CacheHash, V, const N: usize> Cache<K, V, N> {
    // TODO: Rework this implementation to be less simple. The replacement strategy shown here
    // should be tweaked to be more balanced, and of course, fixed-probing should be explored
    // (probing up to some number H of buckets, and then simply replacing)
    pub fn insert(&mut self, key: &K, value: V) {
        let hash = key.hash();

        self.data[hash as usize % N] = Some(Entry { value, hash });
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = key.hash();
        let entry = &self.data[hash as usize % self.data.len()];

        entry.as_ref().and_then(|entry| {
            if entry.hash == hash {
                Some(&entry.value)
            } else {
                None
            }
        })
    }
}

impl<K: CacheHash, V, const N: usize> Cache<K, V, N> {
    pub fn new() -> Self {
        Self {
            data: array::from_fn(|_| None),
            _marker: PhantomData,
        }
    }
}

impl<K: CacheHash, V, const N: usize> Default for Cache<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}
