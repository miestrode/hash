use rustifact::ToTokenStream;

use crate::BitBoard;

#[derive(Clone, Copy, ToTokenStream)]
pub struct Metadata {
    pub offset: usize,
    pub mask: BitBoard,
    pub magic: u64,
    pub shift: usize,
}

impl Metadata {
    pub fn create_local_index(&self, subset: BitBoard) -> usize {
        let relevant_subset = subset & self.mask;
        let hash = relevant_subset.0.wrapping_mul(self.magic);
        (hash >> self.shift) as usize
    }

    pub fn create_global_index(&self, subset: BitBoard) -> usize {
        self.offset + self.create_local_index(subset)
    }
}
