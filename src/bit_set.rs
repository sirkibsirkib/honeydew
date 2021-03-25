use crate::game::room::TOT_CELLS;
use {
    crate::{prelude::*, rng::Rng},
    core::iter::FromIterator,
};

/*
This is an unfortunate case of coupling. What we REALLY want here
is the use of constant generics, to make INDICES a type parameter.
*/
pub const INDICES: u16 = TOT_CELLS;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BitIndex(pub(crate) u16); // invariant: < INDICES

pub struct BitSet {
    // invariant: bits outside of range 0..INDICES are zero
    words: [usize; Self::WORDS as usize],
}
pub struct BitSetIter<'a> {
    bit_set: &'a BitSet,
    cached_word: usize,
    next_idx_of: u16,
}
struct SplitBitIndex {
    idx_in: usize, // invariant: < BitSet::WORD_SIZE
    idx_of: usize, // invariant: < BitSet::WORDS
}
///////////////////////////////////////////////////////////////
impl Default for BitSet {
    fn default() -> Self {
        Self { words: [0; Self::WORDS as usize] }
    }
}
impl SplitBitIndex {
    #[inline]
    fn unsplit(self) -> BitIndex {
        BitIndex(self.idx_of as u16 * BitSet::WORD_SIZE + self.idx_in as u16)
    }
}
const fn div_round_up(x: u16, y: u16) -> u16 {
    (x + y - 1) / y
}

impl FromIterator<BitIndex> for BitSet {
    fn from_iter<I: IntoIterator<Item = BitIndex>>(iter: I) -> Self {
        let mut bs = BitSet::default();
        for i in iter {
            bs.insert(i);
        }
        bs
    }
}

impl BitIndex {
    pub const DOMAIN: Range<u16> = 0..INDICES;
    #[inline]
    fn split(self) -> SplitBitIndex {
        SplitBitIndex {
            idx_of: (self.0 / BitSet::WORD_SIZE) as usize,
            idx_in: (self.0 % BitSet::WORD_SIZE) as usize,
        }
    }
    pub fn random(rng: &mut Rng) -> Self {
        Self(rng.fastrand_rng.u16(Self::DOMAIN))
    }
}

impl BitSet {
    const WORD_SIZE: u16 = core::mem::size_of::<usize>() as u16 * 8;
    const WORDS: u16 = div_round_up(INDICES, Self::WORD_SIZE);
    fn word_and_mask(&self, bit_index: BitIndex) -> (usize, usize) {
        let SplitBitIndex { idx_of, idx_in } = bit_index.split();
        let word = unsafe {
            // safe! relies on invariant
            *self.words.get_unchecked(idx_of)
        };
        (word, 1 << idx_in)
    }
    fn word_and_mask_mut(&mut self, bit_index: BitIndex) -> (&mut usize, usize) {
        let SplitBitIndex { idx_of, idx_in } = bit_index.split();
        let word = unsafe {
            // safe! relies on invariant
            self.words.get_unchecked_mut(idx_of)
        };
        (word, 1 << idx_in)
    }
    fn restore_invariant(&mut self) {
        const DEAD_MSB: u16 = BitSet::WORDS * BitSet::WORD_SIZE - INDICES;
        if DEAD_MSB > 0 {
            const LAST_WORD_IDX: u16 = BitSet::WORDS - 1;
            self.words[LAST_WORD_IDX as usize] &= !0 >> DEAD_MSB as usize;
        }
    }
    pub fn full() -> Self {
        let mut me = Self::default();
        me.set_all(true);
        me
    }
    pub fn contains(&self, bit_index: BitIndex) -> bool {
        let (word, mask) = self.word_and_mask(bit_index);
        word & mask != 0
    }
    pub fn insert(&mut self, bit_index: BitIndex) -> bool {
        let (word, mask) = self.word_and_mask_mut(bit_index);
        let was = *word;
        *word |= mask;
        was != *word
    }
    pub fn remove(&mut self, bit_index: BitIndex) -> bool {
        let (word, mask) = self.word_and_mask_mut(bit_index);
        let was = *word;
        *word &= !mask;
        was != *word
    }
    pub fn set_all(&mut self, set: bool) {
        let set_word = if set { !0 } else { 0 };
        for word in self.words.iter_mut() {
            *word = set_word;
        }
        self.restore_invariant()
    }
    pub fn iter(&self) -> BitSetIter {
        BitSetIter { bit_set: self, cached_word: 0, next_idx_of: 0 }
    }
    pub fn len(&self) -> u16 {
        let s: u32 = self.words.iter().copied().map(usize::count_ones).sum();
        s as u16
    }
}
impl Iterator for BitSetIter<'_> {
    type Item = BitIndex;
    fn next(&mut self) -> Option<Self::Item> {
        while self.cached_word == 0 && self.next_idx_of < BitSet::WORDS {
            // try fill the cached word
            self.cached_word = self.bit_set.words[self.next_idx_of as usize];
            self.next_idx_of += 1;
        }
        if self.cached_word == 0 {
            None
        } else {
            let split_bit_index = SplitBitIndex {
                idx_of: self.next_idx_of as usize - 1,
                idx_in: self.cached_word.trailing_zeros() as usize, // certainly < Self::WORD_SIZE
            };
            // we are removing the `idx_in`th bit in the `idx_of`th word.
            self.cached_word &= !(1 << split_bit_index.idx_in);
            Some(split_bit_index.unsplit())
        }
    }
}
