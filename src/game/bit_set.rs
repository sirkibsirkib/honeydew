use {
    crate::{basic::*, rng::Rng},
    core::iter::FromIterator,
};

pub const W: u8 = 8;
pub const H: u8 = 8;
pub const INDICES: u16 = W as u16 * H as u16;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Index(u16 /* invariant: < INDICES */);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Coord {
    y: u8, // invariant: < H
    x: u8, // invariant: < W
}

pub struct BitSet {
    // invariant: bits outside of range 0..INDICES are zero
    words: [usize; Self::WORDS as usize],
}
pub struct BitSetIter<'a> {
    bit_set: &'a BitSet,
    cached_word: usize,
    next_idx_of: u16,
}
struct SplitIndex {
    idx_in: usize, // invariant: < BitSet::WORD_SIZE
    idx_of: usize, // invariant: < BitSet::WORDS
}
///////////////////////////////////////////////////////////////
impl Default for BitSet {
    fn default() -> Self {
        Self { words: [0; Self::WORDS as usize] }
    }
}
impl SplitIndex {
    #[inline]
    fn unsplit(self) -> Index {
        Index(self.idx_of as u16 * BitSet::WORD_SIZE + self.idx_in as u16)
    }
}
const fn div_round_up(x: u16, y: u16) -> u16 {
    (x + y - 1) / y
}

impl FromIterator<Index> for BitSet {
    fn from_iter<I: IntoIterator<Item = Index>>(iter: I) -> Self {
        let mut bs = BitSet::default();
        for i in iter {
            bs.insert(i);
        }
        bs
    }
}
impl FromIterator<Coord> for BitSet {
    fn from_iter<I: IntoIterator<Item = Coord>>(iter: I) -> Self {
        Self::from_iter(iter.into_iter().map(Into::<Index>::into))
    }
}

impl Index {
    const DOMAIN_RANGE: Range<u16> = 0..INDICES;
    #[inline]
    fn split(self) -> SplitIndex {
        SplitIndex {
            idx_of: (self.0 / BitSet::WORD_SIZE) as usize,
            idx_in: (self.0 % BitSet::WORD_SIZE) as usize,
        }
    }
    fn iter_domain() -> impl Iterator<Item = Self> {
        (Self::DOMAIN_RANGE).map(Index)
    }
    fn random(rng: &mut Rng) -> Self {
        Self(rng.fastrand_rng.u16(Self::DOMAIN_RANGE))
    }
}
impl Into<Coord> for Index {
    fn into(self) -> Coord {
        Coord { x: (self.0 % W as u16) as u8, y: (self.0 / H as u16) as u8 }
    }
}

impl BitSet {
    const WORD_SIZE: u16 = core::mem::size_of::<usize>() as u16 * 8;
    const WORDS: u16 = div_round_up(INDICES, Self::WORD_SIZE);
    fn word_and_mask(&self, bit_index: Index) -> (usize, usize) {
        let SplitIndex { idx_of, idx_in } = bit_index.split();
        let word = unsafe {
            // safe! relies on invariant
            *self.words.get_unchecked(idx_of)
        };
        (word, 1 << idx_in)
    }
    fn word_and_mask_mut(&mut self, bit_index: Index) -> (&mut usize, usize) {
        let SplitIndex { idx_of, idx_in } = bit_index.split();
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
    pub fn contains(&self, bit_index: Index) -> bool {
        let (word, mask) = self.word_and_mask(bit_index);
        word & mask != 0
    }
    pub fn insert(&mut self, bit_index: Index) -> bool {
        let (word, mask) = self.word_and_mask_mut(bit_index);
        let was = *word;
        *word |= mask;
        was != *word
    }
    pub fn remove(&mut self, bit_index: Index) -> bool {
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
    pub fn add_set(&mut self, other: &Self) {
        for (mine, theirs) in self.words.iter_mut().zip(other.words.iter()) {
            *mine |= theirs;
        }
    }
    pub fn remove_set(&mut self, other: &Self) {
        for (mine, theirs) in self.words.iter_mut().zip(other.words.iter()) {
            *mine |= theirs;
        }
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
    type Item = Index;
    fn next(&mut self) -> Option<Self::Item> {
        while self.cached_word == 0 && self.next_idx_of < BitSet::WORDS {
            // try fill the cached word
            self.cached_word = self.bit_set.words[self.next_idx_of as usize];
            self.next_idx_of += 1;
        }
        if self.cached_word == 0 {
            None
        } else {
            let split_bit_index = SplitIndex {
                idx_of: self.next_idx_of as usize - 1,
                idx_in: self.cached_word.trailing_zeros() as usize, // certainly < Self::WORD_SIZE
            };
            // we are removing the `idx_in`th bit in the `idx_of`th word.
            self.cached_word &= !(1 << split_bit_index.idx_in);
            Some(split_bit_index.unsplit())
        }
    }
}

impl Into<Vec2> for Coord {
    fn into(self) -> Vec2 {
        Vec2 { x: self.x as f32, y: self.y as f32 }
    }
}
impl Coord {
    pub fn wall_if_stepped(mut self, dir: Direction) -> Coord {
        match dir.sign() {
            Negative => {}
            Positive => match dir.orientation() {
                Horizontal => self.x = (self.x + 1) % W,
                Vertical => self.y = (self.y + 1) % H,
            },
        }
        self
    }
    pub fn random(rng: &mut Rng) -> Self {
        Index::random(rng).into()
    }
    pub const fn xy(self) -> [u8; 2] {
        [self.x, self.y]
    }
    #[inline]
    pub fn new_checked([x, y]: [u8; 2]) -> Option<Self> {
        if x < W && y < H {
            // invariant established
            Some(Self { x, y })
        } else {
            None
        }
    }
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        Index::iter_domain().map(Into::into)
    }
    pub fn iter_domain_lexicographic(
    ) -> impl Iterator<Item = impl Iterator<Item = Self> + Clone> + Clone {
        (0..H).map(|y| (0..W).map(move |x| Coord { x, y }))
    }
    pub fn stepped(mut self, direction: Direction) -> Self {
        let update = move |value: &mut u8| match direction.sign() {
            Positive => *value += 1,
            Negative => *value -= 1,
        };
        let update_and_correct = |value: &mut u8, bound: u8| {
            update(value);
            *value %= bound;
        };
        match direction.orientation() {
            Vertical => update_and_correct(&mut self.y, H),
            Horizontal => update_and_correct(&mut self.x, W),
        }
        self
    }
}

impl Into<Index> for Coord {
    fn into(self) -> Index {
        Index(self.y as u16 * W as u16 + self.x as u16)
    }
}
