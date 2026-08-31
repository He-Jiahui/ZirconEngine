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
        self.translations.clear();
        self.rotations.clear();
        self.scales.clear();
        self.weights.clear();
        self.translations.resize(joint_count, Vec3::ZERO);
        self.rotations.resize(joint_count, Quat::IDENTITY);
        self.scales.resize(joint_count, Vec3::ONE);
        self.weights.resize(joint_count, 0.0);
    }

    pub(in crate::evaluation) fn clear(&mut self) {
        self.translations.clear();
        self.rotations.clear();
        self.scales.clear();
        self.weights.clear();
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::math::{Quat, Transform, Vec3};

    use super::PoseBuffer;

    #[test]
    fn optimization_batch_20260830cd_pose_buffer_reset_restores_defaults_without_prior_clear() {
        let mut pose = PoseBuffer::new(2);
        pose.set_transform(
            0,
            Transform {
                translation: Vec3::splat(3.0),
                rotation: Quat::from_rotation_y(0.5),
                scale: Vec3::splat(2.0),
            },
        )
        .unwrap();
        pose.set_weight(0, 0.25).unwrap();

        pose.reset(2);

        assert_eq!(pose.transform(0), Some(Transform::default()));
        assert_eq!(pose.weight(0), Some(0.0));
        assert_eq!(pose.len(), 2);
    }

    #[test]
    fn optimization_batch_20260830cd_pose_buffer_reset_initializes_each_row_once() {
        let source = include_str!("storage.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        let reset_start = production
            .find("    pub(in crate::evaluation) fn reset(")
            .expect("reset owner");
        let reset_end = production[reset_start..]
            .find("    pub(in crate::evaluation) fn clear(")
            .map(|offset| reset_start + offset)
            .expect("reset owner boundary");
        let reset = &production[reset_start..reset_end];

        assert_eq!(reset.matches(".clear();").count(), 4);
        assert_eq!(reset.matches(".resize(").count(), 4);
        assert!(!reset.contains(".fill("));
    }

    #[test]
    #[ignore = "Release-only Runtime170 performance contract"]
    fn optimization_batch_20260830cd_pose_buffer_single_initialization_p95() {
        const ROWS: usize = 128;
        const ITERATIONS: usize = 20_000;
        const SAMPLES: usize = 17;
        let mut legacy_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for sample in 0..SAMPLES {
            let legacy = || {
                let mut rows = Vec::with_capacity(ROWS);
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    rows.clear();
                    rows.resize(ROWS, 0_u64);
                    rows.fill(0);
                    black_box(&rows);
                }
                started.elapsed().as_nanos()
            };
            let optimized = || {
                let mut rows = Vec::with_capacity(ROWS);
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    rows.clear();
                    rows.resize(ROWS, 0_u64);
                    black_box(&rows);
                }
                started.elapsed().as_nanos()
            };
            if sample % 2 == 0 {
                legacy_samples.push(legacy());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                legacy_samples.push(legacy());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "RUNTIME170_POSE_BUFFER_SINGLE_INIT_BENCH_V1 baseline_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
            "expected single initialization to reduce P95 by at least 25%: baseline={legacy_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
    }
}
