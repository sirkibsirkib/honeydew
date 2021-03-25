use crate::prelude::*;
use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct WrapInt(i16);

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct WrapVec2(DimMap<WrapInt>);
///////////////////////////////////////////
impl WrapInt {
    pub fn distance_from_zero(self) -> u16 {
        unsafe { core::mem::transmute(self.0.wrapping_abs()) }
    }
}
impl From<i16> for WrapInt {
    #[inline(always)]
    fn from(x: i16) -> Self {
        Self(x)
    }
}
impl Into<i16> for WrapInt {
    #[inline(always)]
    fn into(self) -> i16 {
        self.0
    }
}
impl From<u16> for WrapInt {
    #[inline(always)]
    fn from(x: u16) -> Self {
        Self(unsafe { core::mem::transmute(x) })
    }
}
impl Into<u16> for WrapInt {
    #[inline(always)]
    fn into(self) -> u16 {
        unsafe { core::mem::transmute(self.0) }
    }
}

impl Neg for WrapInt {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl<T> Add<T> for WrapInt
where
    Self: From<T>,
{
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: T) -> Self {
        let [a, b]: [i16; 2] = [self.into(), Self::from(rhs).into()];
        From::from(a.wrapping_add(b))
    }
}

impl<T> Sub<T> for WrapInt
where
    Self: From<T>,
{
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: T) -> Self {
        let [a, b]: [i16; 2] = [self.into(), Self::from(rhs).into()];
        From::from(a.wrapping_sub(b))
    }
}

impl<T> AddAssign<T> for WrapInt
where
    Self: Add<T, Output = Self>,
{
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs;
    }
}

impl<T> SubAssign<T> for WrapInt
where
    Self: Sub<T, Output = Self>,
{
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - rhs;
    }
}
