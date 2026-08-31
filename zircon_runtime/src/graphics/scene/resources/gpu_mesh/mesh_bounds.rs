use crate::core::math::Vec3;

pub(super) struct MeshBoundsAccumulator {
    min: Vec3,
    max: Vec3,
}

impl Default for MeshBoundsAccumulator {
    fn default() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
}

impl MeshBoundsAccumulator {
    pub(super) fn include_position(&mut self, position: [f32; 3]) {
        let position = Vec3::from_array(position);
        self.min = self.min.min(position);
        self.max = self.max.max(position);
    }

    pub(super) fn finish(self) -> (Vec3, Vec3) {
        if !self.min.is_finite() || !self.max.is_finite() {
            (Vec3::ZERO, Vec3::ZERO)
        } else {
            (self.min, self.max)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MeshBoundsAccumulator;
    use crate::core::math::Vec3;

    #[test]
    fn accumulator_preserves_bounds_and_invalid_input_fallback() {
        let mut bounds = MeshBoundsAccumulator::default();
        bounds.include_position([-2.0, 4.0, 1.0]);
        bounds.include_position([3.0, -1.0, 5.0]);
        assert_eq!(
            bounds.finish(),
            (
                Vec3::from_array([-2.0, -1.0, 1.0]),
                Vec3::from_array([3.0, 4.0, 5.0])
            )
        );

        let mut invalid = MeshBoundsAccumulator::default();
        invalid.include_position([f32::NAN, 0.0, 0.0]);
        assert_eq!(invalid.finish(), (Vec3::ZERO, Vec3::ZERO));
    }
}
