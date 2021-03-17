use {
    crate::rng::Rng,
    core::ops::{Mul, Not},
};
pub use {core::ops::Range, enum_map::EnumMap, Direction::*, Orientation::*, Sign::*};

////////////////////////////////
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, enum_map::Enum)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug, enum_map::Enum)]
pub enum Sign {
    Positive,
    Negative,
}

/////////////////////////
impl Mul<f32> for Sign {
    type Output = f32;
    fn mul(self, rhs: f32) -> <Self as Mul<f32>>::Output {
        match self {
            Positive => rhs,
            Negative => -rhs,
        }
    }
}
impl Not for Orientation {
    type Output = Self;
    fn not(self) -> <Self as Not>::Output {
        match self {
            Vertical => Horizontal,
            Horizontal => Vertical,
        }
    }
}

impl Orientation {
    pub fn iter_domain() -> impl Iterator<Item = Self> {
        [Horizontal, Vertical].iter().copied()
    }
    pub fn random(rng: &mut Rng) -> Self {
        if rng.gen_bool() {
            Horizontal
        } else {
            Vertical
        }
    }
    pub const fn vec3_index(self) -> usize {
        match self {
            Horizontal => 0,
            Vertical => 1,
        }
    }
}

impl Direction {
    pub const fn new(ori: Orientation, sign: Sign) -> Self {
        match (ori, sign) {
            (Horizontal, Negative) => Left,
            (Horizontal, Positive) => Right,
            (Vertical, Negative) => Up,
            (Vertical, Positive) => Down,
        }
    }
    pub const fn orientation(self) -> Orientation {
        match self {
            Up | Down => Orientation::Vertical,
            Left | Right => Orientation::Horizontal,
        }
    }
    pub const fn sign(self) -> Sign {
        match self {
            Up | Left => Negative,
            Down | Right => Positive,
        }
    }
}
