use crate::basic::*;
use crate::wrap_fields::wrap_int::WrapInt;
use core::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct WrapVec2([WrapInt; 2]);

///////////////////////////////////////////

impl<T> From<[T; 2]> for WrapVec2
where
    WrapInt: From<T>,
{
    #[inline(always)]
    fn from([x, y]: [T; 2]) -> Self {
        Self([From::from(x), From::from(y)])
    }
}
impl<T> Into<[T; 2]> for WrapVec2
where
    WrapInt: Into<T>,
{
    #[inline(always)]
    fn into(self) -> [T; 2] {
        let Self([x, y]) = self;
        [x.into(), y.into()]
    }
}

impl Index<Dim> for WrapVec2 {
    type Output = WrapInt;
    #[inline(always)]
    fn index(&self, dim: Dim) -> &WrapInt {
        &self.0[dim.vec_index()]
    }
}
impl IndexMut<Dim> for WrapVec2 {
    #[inline(always)]
    fn index_mut(&mut self, dim: Dim) -> &mut WrapInt {
        &mut self.0[dim.vec_index()]
    }
}

impl<T> Add<T> for WrapVec2
where
    Self: From<T>,
{
    type Output = Self;
    #[inline]
    fn add(mut self, rhs: T) -> Self {
        let rhs = Self::from(rhs);
        for dim in Dim::iter_domain() {
            self[dim] += rhs[dim];
        }
        self
    }
}

impl<T> Sub<T> for WrapVec2
where
    Self: From<T>,
{
    type Output = Self;
    #[inline]
    fn sub(mut self, rhs: T) -> Self {
        let rhs = Self::from(rhs);
        for dim in Dim::iter_domain() {
            self[dim] -= rhs[dim];
        }
        self
    }
}
impl<T> Mul<T> for WrapVec2
where
    Self: From<T>,
{
    type Output = Self;
    #[inline]
    fn mul(mut self, rhs: T) -> Self {
        let rhs = Self::from(rhs);
        for dim in Dim::iter_domain() {
            self[dim] *= rhs[dim];
        }
        self
    }
}

impl<T> Div<T> for WrapVec2
where
    Self: From<T>,
{
    type Output = Self;
    #[inline]
    fn div(mut self, rhs: T) -> Self {
        let rhs = Self::from(rhs);
        for dim in Dim::iter_domain() {
            self[dim] *= rhs[dim];
        }
        self
    }
}

impl<T> AddAssign<T> for WrapVec2
where
    Self: Add<T, Output = Self>,
{
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs;
    }
}

impl<T> SubAssign<T> for WrapVec2
where
    Self: Sub<T, Output = Self>,
{
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - rhs;
    }
}

impl<T> MulAssign<T> for WrapVec2
where
    Self: Mul<T, Output = Self>,
{
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs;
    }
}

impl<T> DivAssign<T> for WrapVec2
where
    Self: Div<T, Output = Self>,
{
    fn div_assign(&mut self, rhs: T) {
        *self = *self / rhs;
    }
}
