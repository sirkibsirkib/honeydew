use crate::basic::*;
use core::fmt::Debug;
use core::ops::{Index, IndexMut};
type ElementStorage = u16;
type UserElement = ElementStorage; // same type but in range 0..(1<<USED_ELEMENT_STORAGE_MSB)

const ELEMENT_STORAGE_BITS: usize = core::mem::size_of::<ElementStorage>() * 8;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Element(ElementStorage);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Point {
    map: EnumMap<Orientation, Element>,
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.0 >> Self::UNUSED_LSB).fmt(f)
    }
}
impl Element {
    const USED_MSB: usize = 2;
    const UNUSED_LSB: usize = ELEMENT_STORAGE_BITS - Self::USED_MSB;
    const STORAGE_ONE: ElementStorage = 1 << Self::UNUSED_LSB;
    pub const ONE: Self = Self(Self::STORAGE_ONE);
    pub const ZERO: Self = Self(0);

    pub fn iter_domain() -> impl Iterator<Item = Self> {
        (0..Self::USED_MSB as ElementStorage).map(|x| Self(x << Self::UNUSED_LSB))
    }
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(Self::STORAGE_ONE);
    }
    pub fn decrement(&mut self) {
        self.0 = self.0.wrapping_sub(Self::STORAGE_ONE);
    }
    pub fn from_raw(user_element: UserElement) -> Self {
        Self(user_element << Self::UNUSED_LSB)
    }
    pub fn into_raw(self) -> UserElement {
        self.0 >> Self::UNUSED_LSB
    }
    pub fn added(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }
    pub fn add(&mut self, rhs: Self) {
        *self = self.added(rhs);
    }
    pub fn added_raw(&mut self, rhs: UserElement) -> Self {
        self.added(Self::from_raw(rhs))
    }
    pub fn add_raw(&mut self, rhs: UserElement) {
        *self = self.added_raw(rhs)
    }
}

pub fn new_orientation_enum_map_with<T>(
    mut func: impl FnMut(Orientation) -> T,
) -> EnumMap<Orientation, T> {
    enum_map::enum_map! {
        Horizontal => func(Horizontal),
        Vertical => func(Vertical),
    }
}

impl Point {
    pub const ZERO: Self = unsafe { core::mem::transmute([0 as ElementStorage; 2]) };
    pub fn from_raw([x, y]: [UserElement; 2]) -> Self {
        Self {
            map: enum_map::enum_map! {
                Horizontal => Element::from_raw(x),
                Vertical => Element::from_raw(y),
            },
        }
    }
    pub fn into_raw(self) -> [UserElement; 2] {
        [self.map[Horizontal].into_raw(), self.map[Vertical].into_raw()]
    }
    pub fn added(self, rhs: Self) -> Self {
        Self { map: new_orientation_enum_map_with(move |ori| self.map[ori].added(rhs.map[ori])) }
    }
    pub fn add(&mut self, rhs: Self) {
        *self = self.added(rhs);
    }
}
impl Index<Orientation> for Point {
    type Output = Element;

    #[inline]
    fn index(&self, ori: Orientation) -> &Element {
        &self.map[ori]
    }
}

impl IndexMut<Orientation> for Point {
    #[inline]
    fn index_mut(&mut self, ori: Orientation) -> &mut Element {
        &mut self.map[ori]
    }
}
