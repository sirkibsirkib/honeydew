use {
    crate::rng::Rng,
    core::ops::{Mul, Not},
};
pub use {
    core::{cmp::Ordering, f32::consts::PI as PI_F32, ops::Range},
    enum_map::EnumMap,
    gfx_2020::{glam::Vec2Swizzles, Mat4, Vec2, Vec3},
    ordered_float::OrderedFloat,
    std::collections::HashSet,
    Dim::*,
    Direction::*,
    Sign::*,
};

pub fn iter_pairs<T>(slice: &[T]) -> impl Iterator<Item = [&T; 2]> {
    (0..slice.len() - 1).flat_map(move |left| {
        (left + 1..slice.len())
            .map(move |right| unsafe { [slice.get_unchecked(left), slice.get_unchecked(right)] })
    })
}
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
////////////////////////////////
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, enum_map::Enum)]
pub enum Dim {
    X,
    Y,
}

#[derive(Copy, Clone, Debug, enum_map::Enum)]
pub enum Sign {
    Positive,
    Negative,
}

/////////////////////////
impl Sign {
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        [Positive, Negative].iter().copied()
    }
}
impl Mul<f32> for Sign {
    type Output = f32;
    fn mul(self, rhs: f32) -> <Self as Mul<f32>>::Output {
        match self {
            Positive => rhs,
            Negative => -rhs,
        }
    }
}
impl Not for Dim {
    type Output = Self;
    fn not(self) -> <Self as Not>::Output {
        match self {
            Y => X,
            X => Y,
        }
    }
}

impl Dim {
    pub fn sign(self, sign: Sign) -> Direction {
        Direction::new(self, sign)
    }
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        [X, Y].iter().copied()
    }
    pub fn random(rng: &mut Rng) -> Self {
        if rng.gen_bool() {
            X
        } else {
            Y
        }
    }
    pub const fn vec_index(self) -> usize {
        match self {
            X => 0,
            Y => 1,
        }
    }
}

impl Direction {
    pub const fn new(dim: Dim, sign: Sign) -> Self {
        match (dim, sign) {
            (X, Negative) => Left,
            (X, Positive) => Right,
            (Y, Negative) => Up,
            (Y, Positive) => Down,
        }
    }
    pub const fn dim(self) -> Dim {
        match self {
            Up | Down => Y,
            Left | Right => X,
        }
    }
    pub const fn sign(self) -> Sign {
        match self {
            Up | Left => Negative,
            Down | Right => Positive,
        }
    }
}
