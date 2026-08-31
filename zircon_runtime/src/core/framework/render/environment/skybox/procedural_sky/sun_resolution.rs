use crate::core::math::{Real, Vec4};

use super::{
    ProceduralSkyParams, ResolvedProceduralSun, PROCEDURAL_SKY_DEFAULT_SUN_ANGULAR_RADIUS_RADIANS,
    PROCEDURAL_SKY_MAX_SUN_ANGULAR_RADIUS_RADIANS, PROCEDURAL_SKY_MIN_SUN_ANGULAR_RADIUS_RADIANS,
    PROCEDURAL_SKY_MIN_SUN_DIRECTION_LENGTH_SQUARED, PROCEDURAL_SKY_SUN_INNER_RADIUS_SCALE,
};

impl ProceduralSkyParams {
    pub(crate) fn resolved_sun(&self) -> ResolvedProceduralSun {
        let intensity = if self.sun_intensity.is_finite() {
            self.sun_intensity.max(0.0)
        } else {
            0.0
        };
        let direction_x = f64::from(self.sun_direction.x);
        let direction_y = f64::from(self.sun_direction.y);
        let direction_z = f64::from(self.sun_direction.z);
        let direction_length_squared =
            direction_x * direction_x + direction_y * direction_y + direction_z * direction_z;
        if intensity <= 0.0
            || !direction_length_squared.is_finite()
            || direction_length_squared
                <= f64::from(PROCEDURAL_SKY_MIN_SUN_DIRECTION_LENGTH_SQUARED)
        {
            return ResolvedProceduralSun::default();
        }

        let inverse_direction_length = direction_length_squared.sqrt().recip();
        let normalized_direction = Vec4::new(
            (direction_x * inverse_direction_length) as Real,
            (direction_y * inverse_direction_length) as Real,
            (direction_z * inverse_direction_length) as Real,
            1.0,
        );
        let angular_radius = if self.sun_angular_radius_radians.is_finite() {
            self.sun_angular_radius_radians.clamp(
                PROCEDURAL_SKY_MIN_SUN_ANGULAR_RADIUS_RADIANS,
                PROCEDURAL_SKY_MAX_SUN_ANGULAR_RADIUS_RADIANS,
            )
        } else {
            PROCEDURAL_SKY_DEFAULT_SUN_ANGULAR_RADIUS_RADIANS
        };
        ResolvedProceduralSun {
            direction: normalized_direction,
            intensity_and_cosines: Vec4::new(
                intensity,
                angular_radius.cos(),
                (angular_radius * PROCEDURAL_SKY_SUN_INNER_RADIUS_SCALE).cos(),
                0.0,
            ),
        }
    }
}
