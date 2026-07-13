use zircon_runtime::core::math::{Quat, Vec3};

use super::PoseBuffer;
use crate::{MaskWeights, PoseBlendError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoseLayerBlendMode {
    Override,
    Additive,
}

#[derive(Clone, Copy, Debug)]
pub struct PoseLayer<'a> {
    pose: &'a PoseBuffer,
    weight: f32,
    mode: PoseLayerBlendMode,
    mask: Option<&'a MaskWeights>,
}

impl<'a> PoseLayer<'a> {
    pub fn new(pose: &'a PoseBuffer, weight: f32, mode: PoseLayerBlendMode) -> Self {
        Self {
            pose,
            weight,
            mode,
            mask: None,
        }
    }

    pub fn with_mask(mut self, mask: &'a MaskWeights) -> Self {
        self.mask = Some(mask);
        self
    }
}

impl PoseBuffer {
    pub fn blend_layers(&mut self, layers: &[PoseLayer<'_>]) -> Result<(), PoseBlendError> {
        for layer in layers {
            match layer.mode {
                PoseLayerBlendMode::Override => {
                    self.blend_override_masked(layer.pose, layer.weight, layer.mask)?
                }
                PoseLayerBlendMode::Additive => {
                    self.accumulate_additive_masked(layer.pose, layer.weight, layer.mask)?
                }
            }
        }
        Ok(())
    }

    pub fn blend_override(
        &mut self,
        source: &PoseBuffer,
        weight: f32,
    ) -> Result<(), PoseBlendError> {
        self.blend_override_masked(source, weight, None)
    }

    fn blend_override_masked(
        &mut self,
        source: &PoseBuffer,
        weight: f32,
        mask: Option<&MaskWeights>,
    ) -> Result<(), PoseBlendError> {
        validate_blend_inputs(self, source, weight, mask)?;
        for index in 0..self.len() {
            let effective_weight = source.weights[index] * weight * mask_weight(mask, index);
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
        self.accumulate_additive_masked(source, weight, None)
    }

    fn accumulate_additive_masked(
        &mut self,
        source: &PoseBuffer,
        weight: f32,
        mask: Option<&MaskWeights>,
    ) -> Result<(), PoseBlendError> {
        validate_blend_inputs(self, source, weight, mask)?;
        for index in 0..self.len() {
            let effective_weight = source.weights[index] * weight * mask_weight(mask, index);
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
    mask: Option<&MaskWeights>,
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
    if let Some(mask) = mask {
        if destination.len() != mask.len() {
            return Err(PoseBlendError::MaskShapeMismatch {
                pose_len: destination.len(),
                mask_len: mask.len(),
            });
        }
    }
    Ok(())
}

fn mask_weight(mask: Option<&MaskWeights>, index: usize) -> f32 {
    mask.and_then(|mask| mask.weight(index)).unwrap_or(1.0)
}

fn shortest_slerp(left: Quat, mut right: Quat, weight: f32) -> Quat {
    if left.dot(right) < 0.0 {
        right = -right;
    }
    left.slerp(right, weight).normalize()
}
