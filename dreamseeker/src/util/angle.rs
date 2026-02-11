use std::{f32::consts::PI, ops::{Add, AddAssign, Sub, SubAssign}};

use bevy::reflect::Reflect;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Reflect)]
pub struct Angle(f32);

impl Angle {
    pub fn new(radians: f32) -> Self {
        Self(Self::constrained(radians))
    }

    pub fn get(self) -> f32 {
        self.0
    }

    /// Returns the difference between `self` and `other` in radians,
    /// wrapping around `PI` and `-PI`
    pub fn diff(self, other: Self) -> f32 {
        let mut angle_diff = (self.0 - other.0 + PI) % (2.0 * PI);
        if angle_diff < 0.0 {
            angle_diff += 2.0 * PI;
        }
        angle_diff - PI
    }

    /// constrain angle to [-PI, PI)
    fn constrained(mut angle: f32) -> f32 {
        angle = (angle + PI) % (2.0 * PI);
        if angle < 0.0 {
            angle += 2.0 * PI;
        }
        angle - PI
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(Self::constrained(self.0 + rhs.0))
    }
}

impl AddAssign for Angle {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(Self::constrained(self.0 - rhs.0))
    }
}

impl SubAssign for Angle {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

pub trait AsAngle {
    fn as_angle(self) -> Angle;
}

impl AsAngle for f32 {
    fn as_angle(self) -> Angle {
        Angle::new(self)
    }
}