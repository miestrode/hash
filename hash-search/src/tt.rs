use std::{array, mem, ops::Range};

use hash_core::{board::Board, cache::CacheHash};
use portable_atomic::{AtomicU128, Ordering};

use crate::score::Score;

const BYTE_IN_MIB: usize = 1024 * 1024;
const TABLE_ENTRIES: usize = 64 * BYTE_IN_MIB / mem::size_of::<AtomicEntry>();

#[derive(Clone, Copy)]
pub(crate) enum EntryMetadata {
    Exact = 0,
    LowerBound = 1,
    UpperBound = 2,
}

struct AtomicEntry {
    data: AtomicU128,
}

pub(crate) struct Entry {
    pub(crate) evaluation: Score,
    pub(crate) depth: i16,
    pub(crate) metadata: EntryMetadata,
    pub(crate) hash: u64,
}

impl AtomicEntry {
    // Even though it is possible a hash of an item would be absolutely zero, it is highly unlikely
    // and so can be ignored
    pub(crate) fn empty() -> Self {
        Self {
            data: AtomicU128::new(0),
        }
    }

    pub(crate) fn update(&self, hash: u64, evaluation: Score, depth: i16, metadata: EntryMetadata) {
        // The evaluation may be negative and so be subject to sign extension
        self.data.store(
            hash as u128
                ^ ((evaluation.as_int() as u16 as u128) << 64)
                ^ ((depth as u128) << 80)
                ^ ((metadata as u128) << 96),
            Ordering::Release,
        );
    }

    pub(crate) fn to_entry(&self) -> Entry {
        fn slice(value: u128, range: Range<usize>) -> u128 {
            let shift_to_end = 128 - range.end;
            (value << shift_to_end) >> (shift_to_end + range.start)
        }

        let data = self.data.load(Ordering::Relaxed);

        Entry {
            hash: slice(data, 0..64) as u64,
            evaluation: Score::from_int(slice(data, 64..80) as i16),
            depth: slice(data, 80..96) as i16,
            metadata: match slice(data, 96..128) {
                0 => EntryMetadata::Exact,
                1 => EntryMetadata::LowerBound,
                2 => EntryMetadata::UpperBound,
                _ => unreachable!(),
            },
        }
    }
}

pub(crate) struct Table<const N: usize> {
    data: [AtomicEntry; N],
}

impl<const N: usize> Table<N> {
    pub(crate) fn new() -> Self {
        Self {
            data: array::from_fn(|_| AtomicEntry::empty()),
        }
    }

    // TODO: Rework this implementation to be less simple. The replacement strategy shown here
    // should be tweaked to be more balanced, and of course, fixed-probing should be explored
    // (probing up to some number H of buckets, and then simply replacing)
    pub fn insert(&self, key: &Board, evaluation: Score, depth: i16, metadata: EntryMetadata) {
        let hash = key.hash();

        self.data[hash as usize % N].update(hash, evaluation, depth, metadata)
    }

    pub fn get(&self, key: &Board) -> Option<Entry> {
        let hash = key.hash();
        let entry = self.data[hash as usize % self.data.len()].to_entry();

        if entry.hash == hash {
            Some(entry)
        } else {
            None
        }
    }
}

pub(crate) type ConcreteTable = Table<TABLE_ENTRIES>;
