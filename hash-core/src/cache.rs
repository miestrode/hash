pub trait CacheHash {
    fn hash(&self) -> u64;
}

#[derive(Clone, Copy)]
pub struct Entry<T: Copy> {
    value: T,
    hash: u64,
}

#[derive(Copy, Clone)]
pub struct Cache<T: Copy, const N: usize> {
    data: [Option<Entry<T>>; N],
}

impl<T: Copy, const N: usize> Cache<T, N> {
    // TODO: Rework this implementation to be less simple. The replacement strategy shown here
    // should be tweaked to be more balanced, and of course, fixed-probing should be explored
    // (probing up to some number H of buckets, and then simply replacing)
    pub fn insert<K: CacheHash>(&mut self, key: K, value: T) {
        let hash = key.hash();

        self.data[hash as usize % N] = Some(Entry { value, hash });
    }

    pub fn get<K: CacheHash>(&self, key: &K) -> Option<T> {
        let hash = key.hash();
        let entry = self.data[hash as usize % self.data.len()];

        entry.and_then(|entry| {
            if entry.hash == hash {
                Some(entry.value)
            } else {
                None
            }
        })
    }

    pub fn new() -> Self {
        Self { data: [None; N] }
    }
}

impl<T: Copy, const N: usize> Default for Cache<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
