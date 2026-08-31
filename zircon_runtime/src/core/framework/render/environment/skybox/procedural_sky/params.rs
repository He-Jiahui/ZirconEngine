use crate::core::math::{Real, Vec4};

use super::PROCEDURAL_SKY_DEFAULT_SUN_ANGULAR_RADIUS_RADIANS;

pub const PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProceduralSkyParams {
    pub horizon_color: Vec4,
    pub zenith_color: Vec4,
    pub ground_color: Vec4,
    pub sun_direction: Vec4,
    pub sun_color: Vec4,
    pub sun_intensity: Real,
    pub sun_angular_radius_radians: Real,
    pub intensity: Real,
    pub rotation_radians: Real,
    pub source_revision: u64,
}

impl ProceduralSkyParams {
    pub fn default_gradient() -> Self {
        Self {
            horizon_color: Vec4::new(0.16, 0.19, 0.24, 1.0),
            zenith_color: Vec4::new(0.36, 0.46, 0.63, 1.0),
            ground_color: Vec4::new(0.09, 0.11, 0.14, 1.0),
            sun_direction: Vec4::new(0.0, 1.0, 0.0, 0.0),
            sun_color: Vec4::ONE,
            sun_intensity: 0.0,
            sun_angular_radius_radians: PROCEDURAL_SKY_DEFAULT_SUN_ANGULAR_RADIUS_RADIANS,
            intensity: 1.0,
            rotation_radians: 0.0,
            source_revision: PROCEDURAL_SKY_DEFAULT_SOURCE_REVISION,
        }
    }
}

impl Default for ProceduralSkyParams {
    fn default() -> Self {
        Self::default_gradient()
    }
}
