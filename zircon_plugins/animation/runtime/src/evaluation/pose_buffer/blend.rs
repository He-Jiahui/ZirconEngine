use zircon_runtime::core::math::{Quat, Vec3};

use super::PoseBuffer;
use crate::PoseBlendError;

impl PoseBuffer {
    pub fn blend_override(
        &mut self,
        source: &PoseBuffer,
        weight: f32,
    ) -> Result<(), PoseBlendError> {
        validate_blend_inputs(self, source, weight)?;
        for index in 0..self.len() {
            let effective_weight = source.weights[index] * weight;
            self.translations[index] =
                self.translations[index].lerp(source.translations[index], effective_weight);
            self.rotations[index] = shortest_slerp(
                self.rotations[index],
                source.rotations[index],
                effective_weight,
            );
            self.scales[index] = self.scales[index].lerp(source.scales[index], effective_weight);
            self.weights[index] += (1.0 - self.weights[index]) * effective_weight;
        }
        Ok(())
    }

    pub fn accumulate_additive(
        &mut self,
        source: &PoseBuffer,
        weight: f32,
    ) -> Result<(), PoseBlendError> {
        validate_blend_inputs(self, source, weight)?;
        for index in 0..self.len() {
            let effective_weight = source.weights[index] * weight;
            self.translations[index] += source.translations[index] * effective_weight;
            let rotation_delta =
                shortest_slerp(Quat::IDENTITY, source.rotations[index], effective_weight);
            self.rotations[index] = (rotation_delta * self.rotations[index]).normalize();
            self.scales[index] += (source.scales[index] - Vec3::ONE) * effective_weight;
            self.weights[index] = self.weights[index].max(effective_weight);
        }
        Ok(())
    }
}

fn validate_blend_inputs(
    destination: &PoseBuffer,
    source: &PoseBuffer,
    weight: f32,
) -> Result<(), PoseBlendError> {
    if destination.len() != source.len() {
        return Err(PoseBlendError::ShapeMismatch {
            destination_len: destination.len(),
            source_len: source.len(),
        });
    }
    if !weight.is_finite() || !(0.0..=1.0).contains(&weight) {
        return Err(PoseBlendError::InvalidWeight { weight });
    }
    Ok(())
}

fn shortest_slerp(left: Quat, mut right: Quat, weight: f32) -> Quat {
    if left.dot(right) < 0.0 {
        right = -right;
    }
    left.slerp(right, weight).normalize()
}
