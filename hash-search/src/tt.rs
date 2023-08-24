use std::mem;

use hash_core::cache::Cache;

use crate::score::Score;

#[derive(Clone, Copy)]
pub(crate) enum EntryMetadata {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Copy)]
pub(crate) struct Entry {
    pub(crate) evaluation: Score,
    pub(crate) depth: i16,
    pub(crate) metadata: EntryMetadata,
}

const BYTE_IN_MIB: usize = 1024 * 1024;

const TABLE_ENTRIES: usize = 380 * BYTE_IN_MIB / mem::size_of::<Entry>();

pub(crate) type Table = Cache<Entry, TABLE_ENTRIES>;
