use crate::prelude::*;

/*
This is an unfortunate case of coupling. What we REALLY want here
is the use of constant generics, to make INDICES a type parameter.
*/
pub const INDICES: u16 = crate::game::room::TOT_CELL_COUNT;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BitIndex(pub(crate) u16); // invariant: < INDICES

pub struct BitIndexSet {
    // invariant: bits outside of range 0..INDICES are zero
    words: [usize; Self::WORDS as usize],
}
pub struct BitIndexSetIter<'a> {
    bit_set: &'a BitIndexSet,
    cached_word: usize,
    next_idx_of: u16,
}
struct SplitBitIndex {
    idx_in: usize, // invariant: < BitIndexSet::WORD_SIZE
    idx_of: usize, // invariant: < BitIndexSet::WORDS
}

#[derive(Debug, Copy, Clone)]
pub struct FullBitIndexMap<T> {
    data: [T; INDICES as usize],
}
///////////////////////////////////////////////////////////////
impl<T> FullBitIndexMap<T> {
    pub fn new_copied(t: T) -> Self
    where
        T: Copy,
    {
        Self { data: [t; INDICES as usize] }
    }
}
impl<T> Index<BitIndex> for FullBitIndexMap<T> {
    type Output = T;
    fn index(&self, bi: BitIndex) -> &T {
        &self.data[bi.0 as usize]
    }
}
impl<T> IndexMut<BitIndex> for FullBitIndexMap<T> {
    fn index_mut(&mut self, bi: BitIndex) -> &mut T {
        &mut self.data[bi.0 as usize]
    }
}
impl Default for BitIndexSet {
    fn default() -> Self {
        Self { words: [0; Self::WORDS as usize] }
    }
}
impl SplitBitIndex {
    #[inline]
    fn unsplit(self) -> BitIndex {
        BitIndex(self.idx_of as u16 * BitIndexSet::WORD_SIZE + self.idx_in as u16)
    }
}
const fn div_round_up(x: u16, y: u16) -> u16 {
    (x + y - 1) / y
}

impl core::iter::FromIterator<BitIndex> for BitIndexSet {
    fn from_iter<I: IntoIterator<Item = BitIndex>>(iter: I) -> Self {
        let mut bs = BitIndexSet::default();
        for i in iter {
            bs.insert(i);
        }
        bs
    }
}

impl BitIndex {
    #[inline]
    fn split(self) -> SplitBitIndex {
        SplitBitIndex {
            idx_of: (self.0 / BitIndexSet::WORD_SIZE) as usize,
            idx_in: (self.0 % BitIndexSet::WORD_SIZE) as usize,
        }
    }
    #[inline]
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        (0..INDICES).map(Self)
    }
    pub fn random(rng: &mut Rng) -> Self {
        Self(rng.fastrand_rng.u16(0..INDICES))
    }
}

impl BitIndexSet {
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
        const DEAD_MSB: u16 = BitIndexSet::WORDS * BitIndexSet::WORD_SIZE - INDICES;
        if DEAD_MSB > 0 {
            const LAST_WORD_IDX: u16 = BitIndexSet::WORDS - 1;
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
    pub fn iter(&self) -> BitIndexSetIter {
        BitIndexSetIter { bit_set: self, cached_word: 0, next_idx_of: 0 }
    }
    pub fn len(&self) -> u16 {
        let s: u32 = self.words.iter().copied().map(usize::count_ones).sum();
        s as u16
    }
}
impl Iterator for BitIndexSetIter<'_> {
    type Item = BitIndex;
    fn next(&mut self) -> Option<Self::Item> {
        while self.cached_word == 0 && self.next_idx_of < BitIndexSet::WORDS {
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
