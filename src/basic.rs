use {crate::rng::Rng, core::ops::Not};
pub use {core::ops::Range, Direction::*, Orientation::*, Sign::*};

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

impl Not for Orientation {
    type Output = Self;
    fn not(self) -> <Self as Not>::Output {
        match self {
            Self::Vertical => Self::Horizontal,
            Self::Horizontal => Self::Vertical,
        }
    }
}

impl Orientation {
    pub fn random(rng: &mut Rng) -> Self {
        if rng.gen_bool() {
            Self::Horizontal
        } else {
            Self::Vertical
        }
    }
}

impl Direction {
    pub const fn orientation(self) -> Orientation {
        match self {
            Self::Up | Self::Down => Orientation::Vertical,
            Self::Left | Self::Right => Orientation::Horizontal,
        }
    }
    pub const fn sign(self) -> Sign {
        match self {
            Self::Up | Self::Left => Sign::Negative,
            Self::Down | Self::Right => Sign::Positive,
        }
    }
}
