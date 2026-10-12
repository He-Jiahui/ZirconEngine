use crate::core::math::{Real, Vec4};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ResolvedProceduralSun {
    pub(crate) direction: Vec4,
    pub(crate) intensity_and_cosines: Vec4,
}

impl ResolvedProceduralSun {
    pub(crate) fn direction_for_sampling_rotation(self, rotation_radians: Real) -> Vec4 {
        if self.direction.w < 0.5 || rotation_radians == 0.0 || !rotation_radians.is_finite() {
            return self.direction;
        }
        let (sine, cosine) = rotation_radians.sin_cos();
        Vec4::new(
            self.direction.x * cosine + self.direction.z * sine,
            self.direction.y,
            -self.direction.x * sine + self.direction.z * cosine,
            1.0,
        )
    }
}
