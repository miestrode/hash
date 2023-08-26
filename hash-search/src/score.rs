use std::{
    fmt::{Display, Write},
    ops::Div,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Score {
    value: i16,
}

impl Score {
    pub const MATE_MAXIMUM: i16 = i16::MAX;
    pub const SCORE_MAXIMUM: i16 = 200;

    pub const DRAW: Self = Self { value: 0 };
    pub const WORST: Self = Self::from_mate_distance(-1);
    pub const BEST: Self = Self::from_mate_distance(1);

    pub const fn from_mate_distance(distance: i16) -> Self {
        Self {
            value: distance.signum() * Self::MATE_MAXIMUM - distance,
        }
    }

    pub fn from_evaluation(eval: i16) -> Self {
        Self {
            value: eval.clamp(-Self::SCORE_MAXIMUM, Self::SCORE_MAXIMUM),
        }
    }

    pub fn from_int(score: i16) -> Self {
        Self { value: score }
    }

    pub fn flip_in_place(&mut self) {
        self.value *= -1;
    }

    pub fn flip(mut self) -> Self {
        self.flip_in_place();
        self
    }

    pub fn as_int(&self) -> i16 {
        self.value
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.value.abs() > Score::SCORE_MAXIMUM {
            f.write_char('#')?;
            (Score::MATE_MAXIMUM * self.value.signum() - self.value + self.value.signum())
                .div(2)
                .fmt(f)
        } else {
            self.value.fmt(f)
        }
    }
}
