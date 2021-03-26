use {
    crate::{game::room::ROOM_SIZE, prelude::*, rng::Rng},
    core::ops::{Add, Div, Mul, Neg, Not, Sub},
};
////////////////////////////////
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Dim {
    X = 0,
    Y = 1,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Sign {
    Positive = 0,
    Negative = 1,
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct DimMap<T> {
    pub arr: [T; 2],
}
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct SignMap<T> {
    pub arr: [T; 2],
}

/////////////////////////////////////

impl DimMap<WrapInt> {
    pub fn to_screen2(self) -> Vec2 {
        self.map(Into::<u16>::into).to_screen2()
    }
}
impl DimMap<u16> {
    pub fn to_screen2(self) -> Vec2 {
        let f = move |dim| self[dim] as f32 / ROOM_SIZE[dim] as f32;
        Vec2 { x: f(X), y: f(Y) }
    }
}
impl<T> DimMap<T> {
    pub const fn new(arr: [T; 2]) -> Self {
        Self { arr }
    }
    pub const fn new_xy(x: T, y: T) -> Self {
        Self { arr: [x, y] }
    }
    pub fn new_xy_with(mut func: impl FnMut(Dim) -> T) -> Self {
        Self::new_xy(func(X), func(Y))
    }
    pub fn map<N>(self, f: fn(T) -> N) -> DimMap<N> {
        let [zero, one] = self.arr;
        DimMap { arr: [f(zero), f(one)] }
    }

    #[inline]
    pub fn kv_map<N>(&self, f: fn(Dim, &T) -> N) -> DimMap<N>
    where
        N: Default + Copy,
    {
        let mut new = DimMap { arr: [N::default(); 2] };
        for dim in Dim::iter_domain() {
            new[dim] = f(dim, &self[dim]);
        }
        new
    }
}
impl<T: Add<Output = T> + Copy> Add for DimMap<T> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        for dim in Dim::iter_domain() {
            self[dim] = self[dim] + rhs[dim];
        }
        self
    }
}
impl<T: Sub<Output = T> + Copy> Sub for DimMap<T> {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self {
        for dim in Dim::iter_domain() {
            self[dim] = self[dim] - rhs[dim];
        }
        self
    }
}
impl Div<f32> for DimMap<f32> {
    type Output = Self;
    fn div(mut self, rhs: f32) -> Self {
        for dim in Dim::iter_domain() {
            self[dim] = self[dim] / rhs;
        }
        self
    }
}
impl Mul<f32> for DimMap<f32> {
    type Output = Self;
    fn mul(mut self, rhs: f32) -> Self {
        for dim in Dim::iter_domain() {
            self[dim] = self[dim] * rhs;
        }
        self
    }
}
impl<T> Neg for DimMap<T>
where
    T: Neg<Output = T> + Copy,
{
    type Output = Self;
    fn neg(mut self) -> Self {
        for dim in Dim::iter_domain() {
            self[dim] = -self[dim];
        }
        self
    }
}
impl<T> Index<Dim> for DimMap<T> {
    type Output = T;
    fn index(&self, dim: Dim) -> &T {
        &self.arr[dim.map_index()]
    }
}
impl<T> IndexMut<Dim> for DimMap<T> {
    fn index_mut(&mut self, dim: Dim) -> &mut T {
        &mut self.arr[dim.map_index()]
    }
}
impl<T> SignMap<T> {
    pub const fn new(arr: [T; 2]) -> Self {
        Self { arr }
    }
}
impl<T> Index<Sign> for SignMap<T> {
    type Output = T;
    fn index(&self, sign: Sign) -> &T {
        &self.arr[sign.map_index()]
    }
}
impl<T> IndexMut<Sign> for SignMap<T> {
    fn index_mut(&mut self, sign: Sign) -> &mut T {
        &mut self.arr[sign.map_index()]
    }
}

impl Sign {
    const fn map_index(self) -> usize {
        match self {
            Positive => 0,
            Negative => 1,
        }
    }
}
impl Dim {
    const fn map_index(self) -> usize {
        match self {
            X => 0,
            Y => 1,
        }
    }
}
// impl<A, B> Into<DimMap<B>> for DimMap<A> {
//     fn into(self) -> DimMap<B> {
//         let Self { arr: [zero, one] } = self;
//         Self::new([zero.into(), one.into()])
//     }
// }
/////////////////////////

impl<T: Neg<Output = T>> Mul<T> for Sign {
    type Output = T;
    #[inline(always)]
    fn mul(self, rhs: T) -> T {
        match self {
            Positive => rhs,
            Negative => -rhs,
        }
    }
}
impl Not for Dim {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> <Self as Not>::Output {
        match self {
            Y => X,
            X => Y,
        }
    }
}
impl Not for Sign {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> <Self as Not>::Output {
        match self {
            Positive => Negative,
            Negative => Positive,
        }
    }
}

impl Dim {
    pub const DOMAIN: [Self; 2] = [X, Y];

    #[inline(always)]
    pub fn sign(self, sign: Sign) -> Direction {
        Direction::new(self, sign)
    }
    #[inline(always)]
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        Self::DOMAIN.iter().copied()
    }
    pub fn random(rng: &mut Rng) -> Self {
        if rng.gen_bool() {
            X
        } else {
            Y
        }
    }
}

impl Direction {
    #[inline(always)]
    pub const fn new(dim: Dim, sign: Sign) -> Self {
        match (dim, sign) {
            (X, Negative) => Left,
            (X, Positive) => Right,
            (Y, Negative) => Up,
            (Y, Positive) => Down,
        }
    }
    #[inline(always)]
    pub const fn dim(self) -> Dim {
        match self {
            Up | Down => Y,
            Left | Right => X,
        }
    }
    #[inline(always)]
    pub const fn sign(self) -> Sign {
        match self {
            Up | Left => Negative,
            Down | Right => Positive,
        }
    }
}
