use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};

use super::PoseBuffer;
use crate::PoseBufferError;

impl PoseBuffer {
    pub fn set_transform(
        &mut self,
        index: usize,
        transform: Transform,
    ) -> Result<(), PoseBufferError> {
        if index >= self.len() {
            return Err(PoseBufferError::IndexOutOfBounds {
                index,
                len: self.len(),
            });
        }
        if !transform.translation.is_finite()
            || !transform.rotation.is_finite()
            || !transform.scale.is_finite()
        {
            return Err(PoseBufferError::NonFiniteTransform { index });
        }
        if transform.rotation.length_squared() <= Real::EPSILON {
            return Err(PoseBufferError::ZeroLengthRotation { index });
        }

        self.translations[index] = transform.translation;
        self.rotations[index] = transform.rotation.normalize();
        self.scales[index] = transform.scale;
        self.weights[index] = 1.0;
        Ok(())
    }

    pub fn set_weight(&mut self, index: usize, weight: f32) -> Result<(), PoseBufferError> {
        if index >= self.len() {
            return Err(PoseBufferError::IndexOutOfBounds {
                index,
                len: self.len(),
            });
        }
        if !weight.is_finite() || !(0.0..=1.0).contains(&weight) {
            return Err(PoseBufferError::InvalidWeight { index, weight });
        }
        self.weights[index] = weight;
        Ok(())
    }

    pub(in crate::evaluation) fn reset(&mut self, joint_count: usize) {
        self.translations.resize(joint_count, Vec3::ZERO);
        self.rotations.resize(joint_count, Quat::IDENTITY);
        self.scales.resize(joint_count, Vec3::ONE);
        self.weights.resize(joint_count, 0.0);
        self.translations.fill(Vec3::ZERO);
        self.rotations.fill(Quat::IDENTITY);
        self.scales.fill(Vec3::ONE);
        self.weights.fill(0.0);
    }

    pub(in crate::evaluation) fn clear(&mut self) {
        self.translations.clear();
        self.rotations.clear();
        self.scales.clear();
        self.weights.clear();
    }
}
