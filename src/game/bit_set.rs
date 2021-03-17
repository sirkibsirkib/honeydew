const W: u8 = 8;
const H: u8 = 8;
const INDICES: u16 = W as u16 * H as u16;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BitIndex(u16 /* invariant: < INDICES */);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BitCoord {
    y: u8, // invariant: < H
    x: u8, // invariant: < W
}

#[derive(Default)]
pub struct BitSet {
    // invariant: bits outside of range 0..INDICES are zero
    words: [usize; Self::WORDS as usize],
}

///////////////////////////////////////////////////////////////
const fn div_round_up(x: u16, y: u16) -> u16 {
    (x + y - 1) / y
}

impl BitIndex {
    fn iter_domain() -> impl Iterator<Item = Self> {
        (0..INDICES).map(BitIndex)
    }
}
impl Into<BitCoord> for BitIndex {
    fn into(self) -> BitCoord {
        BitCoord { x: (self.0 % W as u16) as u8, y: (self.0 / H as u16) as u8 }
    }
}

impl BitSet {
    const WORD_SIZE: u16 = core::mem::size_of::<usize>() as u16;
    const WORDS: u16 = div_round_up(INDICES, Self::WORD_SIZE);

    #[inline]
    fn index_split(bit_index: BitIndex) -> [usize; 2] {
        [(bit_index.0 / Self::WORD_SIZE) as usize, (bit_index.0 % Self::WORD_SIZE) as usize]
    }
    fn word_and_mask(&self, bit_index: BitIndex) -> (usize, usize) {
        let [idx_of, idx_in] = Self::index_split(bit_index);
        let word = unsafe {
            // safe! relies on invariant
            *self.words.get_unchecked(idx_of)
        };
        (word, 1 << idx_in)
    }
    fn word_and_mask_mut(&mut self, bit_index: BitIndex) -> (&mut usize, usize) {
        let [idx_of, idx_in] = Self::index_split(bit_index);
        let word = unsafe {
            // safe! relies on invariant
            self.words.get_unchecked_mut(idx_of)
        };
        (word, 1 << idx_in)
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
        const DEAD_TAIL_BITS: u16 = BitSet::WORDS * BitSet::WORD_SIZE - INDICES;
        if DEAD_TAIL_BITS > 0 {
            const LAST_WORD_IDX: u16 = BitSet::WORDS - 1;
            self.words[LAST_WORD_IDX as usize] & (!0 << DEAD_TAIL_BITS as usize);
        }
    }
}

impl BitCoord {
    pub fn xy(self) -> [u8; 2] {
        [self.x, self.y]
    }
    pub fn new_checked([x, y]: [u8; 2]) -> Option<Self> {
        if x < W && y < H {
            // invariant established
            Some(Self { x, y })
        } else {
            None
        }
    }
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        BitIndex::iter_domain().map(Into::into)
    }
}

impl Into<BitIndex> for BitCoord {
    fn into(self) -> BitIndex {
        BitIndex(self.y as u16 * W as u16 + self.x as u16)
    }
}
