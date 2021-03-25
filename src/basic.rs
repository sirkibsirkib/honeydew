use core::ops::Add;
use core::ops::Div;
use core::ops::Neg;
use core::ops::Sub;
use {
    crate::{prelude::*, rng::Rng},
    core::ops::{Mul, Not},
};
////////////////////////////////
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

#[derive(Copy, Clone, Debug)]
pub enum Dim {
    X = 0,
    Y = 1,
}

#[derive(Copy, Clone, Debug)]
pub enum Sign {
    Positive = 0,
    Negative = 1,
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Default)]
pub struct DimMap<T> {
    pub arr: [T; 2],
}
impl DimMap<f32> {
    pub fn distance_squared(self, rhs: Self) -> f32 {
        let dx = self[X] - rhs[X];
        let dy = self[Y] - rhs[Y];
        dx * dx + dy * dy
    }
    pub fn extend(self, z: f32) -> Vec3 {
        Vec3::from([self[X], self[Y], z])
    }
    pub fn yx(self) -> Self {
        Self { arr: [self.arr[1], self.arr[0]] }
    }
}
impl<T> DimMap<T> {
    pub fn map<N>(&self, f: fn(&T) -> N) -> DimMap<N> {
        DimMap { arr: [f(&self.arr[0]), f(&self.arr[1])] }
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
impl Neg for DimMap<f32> {
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

#[derive(Debug, Copy, Clone, Default)]
pub struct SignMap<T> {
    pub arr: [T; 2],
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

// #[derive(Debug)]
// pub struct Arr2Map<K: Arr2MapKey, V> {
//     pub arr: [V; 2],
//     _phantom: PhantomData<K>,
// }

// impl<K: Arr2MapKey, V: Clone> Clone for Arr2Map<K, V> {
//     fn clone(&self) -> Self {
//         Self::new([self.arr[0], self.arr[1]])
//     }
// }
// impl<K: Arr2MapKey, V: Copy> Copy for Arr2Map<K, V> {}

// pub trait Arr2MapKey {
//     const ONE_INDEX_FN: fn(Self) -> bool;
// }
// impl<K: Arr2MapKey, V> Arr2Map<K, V> {
//     pub fn new_cloning(both: V) -> Self
//     where
//         V: Clone,
//     {
//         Self::new([both.clone(), both])
//     }
//     pub fn new(arr: [V; 2]) -> Self {
//         Self { arr, _phantom: PhantomData {} }
//     }
// }

// impl<K: Arr2MapKey, V> Index<K> for Arr2Map<K, V> {
//     type Output = V;
//     #[inline]
//     fn index(&self, key: K) -> &V {
//         &self.arr[if K::ONE_INDEX_FN(key) { 1 } else { 0 }]
//     }
// }
// impl<K: Arr2MapKey, V> IndexMut<K> for Arr2Map<K, V> {
//     #[inline]
//     fn index_mut(&mut self, key: K) -> &mut V {
//         &mut self.arr[if K::ONE_INDEX_FN(key) { 1 } else { 0 }]
//     }
// }

// impl Arr2MapKey for Dim {
//     const ONE_INDEX_FN: fn(Self) -> bool = |dim| match dim {
//         X => false,
//         Y => true,
//     };
// }
// impl Arr2MapKey for Sign {
//     const ONE_INDEX_FN: fn(Self) -> bool = |sign| match sign {
//         Negative => false,
//         Positive => true,
//     };
// }

// pub struct DimMap<T>([T; 2]);
/////////////////////////

// impl<T> Index<Dim> for DimMap<T> {
//     type Output = T;
//     fn index(&self, dim: Dim) -> &T {
//         &self.0[dim.vec_index()]
//     }
// }
// impl<T> IndexMut<Dim> for DimMap<T> {
//     fn index_mut(&mut self, dim: Dim) -> &mut T {
//         &mut self.0[dim.vec_index()]
//     }
// }

// pub fn iter_pairs<T>(slice: &[T]) -> impl Iterator<Item = [&T; 2]> {
//     (0..slice.len() - 1).flat_map(move |left| {
//         (left + 1..slice.len())
//             .map(move |right| unsafe { [slice.get_unchecked(left), slice.get_unchecked(right)] })
//     })
// }
pub fn iter_pairs_mut<T>(slice: &mut [T]) -> impl Iterator<Item = [&mut T; 2]> {
    let p = slice.as_mut_ptr();
    (0..slice.len() - 1).flat_map(move |left| {
        (left + 1..slice.len()).map(move |right| unsafe { [&mut *p.add(left), &mut *p.add(right)] })
    })
}
pub fn modulo_difference([a, b]: [f32; 2], modulus: f32) -> f32 {
    // assume positive modulus
    // assume {a, b} in 0..modulus
    let wraps = (a - b).abs() > modulus / 2.;
    if wraps {
        (a + modulus) - b
    } else {
        a - b
    }
}
pub fn modulo_distance([a, b]: [f32; 2], modulus: f32) -> f32 {
    // assumes inputs are in range 0..modulus
    let direct_dist = (a - b).abs();
    (modulus - direct_dist).min(direct_dist)
}

/////////////////////////
impl Sign {
    pub const DOMAIN: [Self; 2] = [Positive, Negative];

    #[inline(always)]
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        Self::DOMAIN.iter().copied()
    }
}
impl Mul<f32> for Sign {
    type Output = f32;
    #[inline(always)]
    fn mul(self, rhs: f32) -> <Self as Mul<f32>>::Output {
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

// impl Arr2MapKey for Dim {
//     #[inline]
//     fn array_index(self) -> usize {
//         match self {
//             X => 0,
//             Y => 1,
//         }
//     }
// }

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
    pub const DOMAIN: [Self; 4] = [Up, Down, Left, Right];

    pub fn iter_domain() -> impl Iterator<Item = Self> {
        Self::DOMAIN.iter().copied()
    }

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
