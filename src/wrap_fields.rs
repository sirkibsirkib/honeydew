use crate::prelude::*;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct WrapInt(i16);

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct WrapVec2(Arr2Map<Dim, WrapInt>);

///////////////////////////////////////
// Associated consts

///////////////////////////////////////
// WrapInt impls

impl WrapInt {
    pub const fn distance_to_zero(self) -> u16 {
        self.0.wrapping_abs() as u16
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
        From::from(Into::<i16>::into(self).wrapping_neg())
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

////////////////////////////////
// WrapVec2 impls
impl WrapVec2 {
    #[inline]
    pub fn extend_to_vec3(self, z: f32) -> Vec3 {
        let f = move |dim| {
            let x: i16 = self[dim].into();
            x as f32
        };
        Vec3::new(f(X), f(Y), z)
    }
    pub fn manhattan_to_zero(self) -> u32 {
        Dim::iter_domain().map(|dim| self[dim].distance_to_zero() as u32).sum()
    }
    pub fn diagonal<T>(x: T) -> Self
    where
        T: Copy,
        WrapInt: From<T>,
    {
        let x = WrapInt::from(x);
        let mut ret = Self::default();
        for dim in Dim::iter_domain() {
            ret[dim] = x;
        }
        ret
    }
    // pub fn div_scalar<T>(mut self, rhs: T) -> Self
    // where
    //     WrapInt: From<T>,
    //     T: Copy,
    // {
    //     for dim in Dim::iter_domain() {
    //         self[dim] /= rhs;
    //     }
    //     self
    // }
}

impl<T: Copy> From<[T; 2]> for WrapVec2
where
    WrapInt: From<T>,
{
    #[inline(always)]
    fn from(arr: [T; 2]) -> Self {
        Self(Arr2Map::new(arr))
    }
}
impl<T> Into<[T; 2]> for WrapVec2
where
    WrapInt: Into<T>,
{
    #[inline(always)]
    fn into(self) -> [T; 2] {
        [self[X].into(), self[Y].into()]
    }
}

impl Index<Dim> for WrapVec2 {
    type Output = WrapInt;
    #[inline(always)]
    fn index(&self, dim: Dim) -> &WrapInt {
        &self.0[dim]
    }
}
impl IndexMut<Dim> for WrapVec2 {
    #[inline(always)]
    fn index_mut(&mut self, dim: Dim) -> &mut WrapInt {
        &mut self.0[dim]
    }
}

impl Neg for WrapVec2 {
    type Output = Self;
    fn neg(mut self) -> Self {
        for dim in Dim::iter_domain() {
            self[dim] = -self[dim];
        }
        self
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
