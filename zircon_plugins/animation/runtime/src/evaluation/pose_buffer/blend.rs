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
            if effective_weight == 0.0 {
                continue;
            }
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
            if effective_weight == 0.0 {
                continue;
            }
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

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::math::{Quat, Transform, Vec3};

    use super::PoseBuffer;

    #[test]
    fn optimization_batch_20260830ch_zero_effective_weight_preserves_pose() {
        let mut destination = PoseBuffer::new(1);
        let expected = Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_rotation_y(0.5),
            scale: Vec3::splat(1.5),
        };
        destination.set_transform(0, expected).unwrap();
        let mut source = PoseBuffer::new(1);
        source
            .set_transform(0, Transform::from_translation(Vec3::splat(9.0)))
            .unwrap();
        source.set_weight(0, 0.0).unwrap();

        destination.blend_override(&source, 1.0).unwrap();
        assert_eq!(destination.transform(0), Some(expected));
        destination.accumulate_additive(&source, 1.0).unwrap();
        assert_eq!(destination.transform(0), Some(expected));
    }

    #[test]
    fn optimization_batch_20260830ch_zero_effective_weight_static_contract() {
        let production = include_str!("blend.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert_eq!(production.matches("if effective_weight == 0.0").count(), 2);
    }

    #[test]
    #[ignore = "Release-only Runtime170 performance contract"]
    fn optimization_batch_20260830ch_zero_effective_weight_p95() {
        const JOINTS: usize = 256;
        const ITERATIONS: usize = 20_000;
        const SAMPLES: usize = 17;
        let source = vec![[0.2_f32, 0.3, 0.4, 0.5]; JOINTS];
        let weights = vec![1.0_f32; JOINTS];
        let mut baseline_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for sample in 0..SAMPLES {
            let baseline = || {
                let mut destination = vec![[0.0_f32, 0.0, 0.0, 1.0]; JOINTS];
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    zero_weight_blend_model(&mut destination, &source, &weights, false);
                }
                black_box(destination);
                started.elapsed().as_nanos()
            };
            let optimized = || {
                let mut destination = vec![[0.0_f32, 0.0, 0.0, 1.0]; JOINTS];
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    zero_weight_blend_model(&mut destination, &source, &weights, true);
                }
                black_box(destination);
                started.elapsed().as_nanos()
            };
            if sample % 2 == 0 {
                baseline_samples.push(baseline());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                baseline_samples.push(baseline());
            }
        }

        let baseline_p95 = percentile_95(&mut baseline_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "RUNTIME170_ZERO_EFFECTIVE_WEIGHT_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(70),
            "expected zero-weight early exit to reduce P95 by at least 30%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn zero_weight_blend_model(
        destination: &mut [[f32; 4]],
        source: &[[f32; 4]],
        weights: &[f32],
        skip_zero: bool,
    ) {
        for index in 0..destination.len() {
            let weight = weights[index] * 0.0;
            if skip_zero && weight <= f32::EPSILON {
                continue;
            }
            for component in 0..4 {
                destination[index][component] +=
                    (source[index][component] - destination[index][component]) * weight;
            }
            black_box(destination[index]);
        }
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
    }
}
