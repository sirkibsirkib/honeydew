use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct WrapInt(i16);

///////////////////////////////////////////

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

impl<T> Mul<T> for WrapInt
where
    Self: From<T>,
{
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: T) -> Self {
        let [a, b]: [i16; 2] = [self.into(), Self::from(rhs).into()];
        From::from(a.wrapping_mul(b))
    }
}

impl<T> Div<T> for WrapInt
where
    Self: From<T>,
{
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: T) -> Self {
        let [a, b]: [i16; 2] = [self.into(), Self::from(rhs).into()];
        From::from(a.wrapping_div(b))
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

impl<T> MulAssign<T> for WrapInt
where
    Self: Mul<T, Output = Self>,
{
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs;
    }
}

impl<T> DivAssign<T> for WrapInt
where
    Self: Div<T, Output = Self>,
{
    fn div_assign(&mut self, rhs: T) {
        *self = *self / rhs;
    }
}
