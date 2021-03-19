use crate::basic::*;
use core::fmt::Debug;
use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Neg;
use core::ops::Sub;
use core::ops::SubAssign;
use core::ops::{Index, IndexMut};
type ElementStorage = u16;
type UserElement = ElementStorage; // same type but in range 0..(1<<USED_ELEMENT_STORAGE_MSB)

const ELEMENT_STORAGE_BITS: usize = core::mem::size_of::<ElementStorage>() * 8;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Element(ElementStorage);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Point {
    map: EnumMap<Dim, Element>,
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        (self.0 >> Self::UNUSED_LSB).fmt(f)
    }
}
impl Neg for Element {
    type Output = Self;
    fn neg(mut self) -> Self {
        self.0 ^= 1 << (ELEMENT_STORAGE_BITS - 1);
        self
    }
}
impl<T> Add<T> for Element
where
    Element: From<T>,
{
    type Output = Self;
    #[inline]
    fn add(self, rhs: T) -> Self {
        let rhs: Self = From::from(rhs);
        Self(self.0.wrapping_add(rhs.0))
    }
}
impl<T> Sub<T> for Element
where
    Element: From<T>,
{
    type Output = Self;
    #[inline]
    fn sub(self, rhs: T) -> Self {
        let rhs: Self = From::from(rhs);
        Self(self.0.wrapping_sub(rhs.0))
    }
}
impl<T> AddAssign<T> for Element
where
    Element: From<T>,
{
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        *self = *self + From::from(rhs);
    }
}
impl<T> SubAssign<T> for Element
where
    Element: From<T>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - From::from(rhs);
    }
}
impl From<UserElement> for Element {
    #[inline]
    fn from(x: UserElement) -> Self {
        Self(x << Self::UNUSED_LSB)
    }
}
impl Into<UserElement> for Element {
    #[inline]
    fn into(self) -> UserElement {
        self.0 >> Self::UNUSED_LSB
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
}

pub fn new_dimentation_enum_map_with<T>(mut func: impl FnMut(Dim) -> T) -> EnumMap<Dim, T> {
    enum_map::enum_map! {
        X => func(X),
        Y => func(Y),
    }
}

impl Point {
    pub const ZERO: Self = unsafe { core::mem::transmute([0 as ElementStorage; 2]) };
    // pub fn from_raw([x, y]: [UserElement; 2]) -> Self {
    //     Self {
    //         map: enum_map::enum_map! {
    //             X => Element::from_raw(x),
    //             Y => Element::from_raw(y),
    //         },
    //     }
    // }
    // pub fn into_raw(self) -> [UserElement; 2] {
    //     [self[X].into_raw(), self[Y].into_raw()]
    // }
}
impl From<[UserElement; 2]> for Point {
    fn from([x, y]: [UserElement; 2]) -> Self {
        Self {
            map: enum_map::enum_map! {
                X => Element::from(x),
                Y => Element::from(y),
            },
        }
    }
}
impl Into<[UserElement; 2]> for Point {
    fn into(self) -> [UserElement; 2] {
        [self[X].into(), self[Y].into()]
    }
}
impl<T> Add<T> for Point
where
    Point: From<T>,
{
    type Output = Self;
    #[inline]
    fn add(self, rhs: T) -> Self {
        let rhs: Self = From::from(rhs);
        Self { map: new_dimentation_enum_map_with(move |dim| self[dim] + rhs[dim]) }
    }
}
impl<T> AddAssign<T> for Point
where
    Point: From<T>,
{
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        *self = *self + From::from(rhs);
    }
}
impl<T> Sub<T> for Point
where
    Point: From<T>,
{
    type Output = Self;
    #[inline]
    fn sub(self, rhs: T) -> Self {
        let rhs: Self = From::from(rhs);
        Self { map: new_dimentation_enum_map_with(move |dim| self[dim] - rhs[dim]) }
    }
}
impl<T> SubAssign<T> for Point
where
    Point: From<T>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - From::from(rhs);
    }
}
impl Neg for Point {
    type Output = Self;
    fn neg(self) -> Self {
        Self { map: new_dimentation_enum_map_with(move |dim| -self[dim]) }
    }
}
impl Index<Dim> for Point {
    type Output = Element;

    #[inline]
    fn index(&self, dim: Dim) -> &Element {
        &self.map[dim]
    }
}

impl IndexMut<Dim> for Point {
    #[inline]
    fn index_mut(&mut self, dim: Dim) -> &mut Element {
        &mut self.map[dim]
    }
}
