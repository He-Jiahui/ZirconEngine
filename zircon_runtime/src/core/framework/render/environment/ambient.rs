use serde::{Deserialize, Serialize};

use crate::core::math::Vec3;

pub const SH_L2_RGB_COEFFICIENT_COUNT: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShL2Rgb(pub [Vec3; SH_L2_RGB_COEFFICIENT_COUNT]);

impl ShL2Rgb {
    pub const ZERO: Self = Self([Vec3::ZERO; SH_L2_RGB_COEFFICIENT_COUNT]);

    pub const fn coefficients(&self) -> &[Vec3; SH_L2_RGB_COEFFICIENT_COUNT] {
        &self.0
    }

    pub fn is_finite(&self) -> bool {
        self.0.iter().all(|coefficient| coefficient.is_finite())
    }
}

impl Default for ShL2Rgb {
    fn default() -> Self {
        Self::ZERO
    }
}
