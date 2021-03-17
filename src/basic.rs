use {crate::rng::Rng, core::ops::Not};

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
