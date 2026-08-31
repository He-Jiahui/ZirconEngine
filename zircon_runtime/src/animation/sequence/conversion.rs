use crate::core::framework::animation::AnimationChannelValueAsset;
use crate::core::framework::animation::{AnimationError, AnimationResult};
use crate::core::framework::scene::ScenePropertyValue;
use crate::core::math::Real;

pub(super) fn scene_property_value_from_channel(
    value: &AnimationChannelValueAsset,
) -> AnimationResult<ScenePropertyValue> {
    match value {
        AnimationChannelValueAsset::Bool(value) => Ok(ScenePropertyValue::Bool(*value)),
        AnimationChannelValueAsset::Integer(value) => {
            Ok(ScenePropertyValue::Integer(*value as i64))
        }
        AnimationChannelValueAsset::Vec2(value) => {
            project_finite_array(value, "vec2", ScenePropertyValue::Vec2)
        }
        AnimationChannelValueAsset::Vec3(value) => {
            project_finite_array(value, "vec3", ScenePropertyValue::Vec3)
        }
        AnimationChannelValueAsset::Vec4(value) => {
            project_finite_array(value, "vec4", ScenePropertyValue::Vec4)
        }
        AnimationChannelValueAsset::Scalar(value) => {
            if value.is_finite() {
                Ok(ScenePropertyValue::Scalar(*value))
            } else {
                Err(non_finite_channel_sample("scalar"))
            }
        }
        AnimationChannelValueAsset::Quaternion(value) => {
            if !value.iter().all(|component| component.is_finite()) {
                return Err(non_finite_channel_sample("quaternion"));
            }
            if !quaternion_array_is_normalizable(value) {
                return Err(AnimationError::ZeroLengthQuaternionChannelSample);
            }
            Ok(ScenePropertyValue::Quaternion(*value))
        }
    }
}

fn project_finite_array<const N: usize>(
    value: &[Real; N],
    sample_kind: &'static str,
    project: impl FnOnce([Real; N]) -> ScenePropertyValue,
) -> AnimationResult<ScenePropertyValue> {
    if value.iter().all(|component| component.is_finite()) {
        Ok(project(*value))
    } else {
        Err(non_finite_channel_sample(sample_kind))
    }
}

fn non_finite_channel_sample(sample_kind: &'static str) -> AnimationError {
    AnimationError::NonFiniteChannelSample { sample_kind }
}

fn quaternion_array_is_normalizable(value: &[Real; 4]) -> bool {
    value
        .iter()
        .map(|component| component * component)
        .sum::<Real>()
        > Real::EPSILON
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_20260831fc_runtime568_single_dispatch_preserves_values_and_errors() {
        for value in [
            AnimationChannelValueAsset::Bool(true),
            AnimationChannelValueAsset::Integer(7),
            AnimationChannelValueAsset::Scalar(1.5),
            AnimationChannelValueAsset::Vec2([1.0, 2.0]),
            AnimationChannelValueAsset::Vec3([1.0, 2.0, 3.0]),
            AnimationChannelValueAsset::Vec4([1.0, 2.0, 3.0, 4.0]),
            AnimationChannelValueAsset::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ] {
            assert_eq!(
                scene_property_value_from_channel(&value),
                legacy_projection(&value)
            );
        }

        assert!(matches!(
            scene_property_value_from_channel(&AnimationChannelValueAsset::Vec3([
                1.0,
                Real::NAN,
                3.0
            ])),
            Err(AnimationError::NonFiniteChannelSample {
                sample_kind: "vec3"
            })
        ));
        assert!(matches!(
            scene_property_value_from_channel(&AnimationChannelValueAsset::Quaternion([0.0; 4])),
            Err(AnimationError::ZeroLengthQuaternionChannelSample)
        ));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260831fc_runtime568_single_dispatch_p95() {
        const SAMPLE_PAIRS: usize = 13;
        const ITERATIONS: u64 = 10_000_000;
        let values = [
            AnimationChannelValueAsset::Scalar(1.0),
            AnimationChannelValueAsset::Vec3([1.0, 2.0, 3.0]),
            AnimationChannelValueAsset::Vec4([1.0, 2.0, 3.0, 4.0]),
            AnimationChannelValueAsset::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ];
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, &values, ITERATIONS));
                optimized.push(measure(true, &values, ITERATIONS));
            } else {
                optimized.push(measure(true, &values, ITERATIONS));
                legacy.push(measure(false, &values, ITERATIONS));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME568_CHANNEL_PROJECTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85));
    }

    fn legacy_projection(
        value: &AnimationChannelValueAsset,
    ) -> AnimationResult<ScenePropertyValue> {
        if !legacy_value_is_finite(value) {
            return Err(non_finite_channel_sample(legacy_value_kind(value)));
        }
        if let AnimationChannelValueAsset::Quaternion(value) = value {
            if !quaternion_array_is_normalizable(value) {
                return Err(AnimationError::ZeroLengthQuaternionChannelSample);
            }
        }
        Ok(match value {
            AnimationChannelValueAsset::Bool(value) => ScenePropertyValue::Bool(*value),
            AnimationChannelValueAsset::Integer(value) => {
                ScenePropertyValue::Integer(*value as i64)
            }
            AnimationChannelValueAsset::Scalar(value) => ScenePropertyValue::Scalar(*value),
            AnimationChannelValueAsset::Vec2(value) => ScenePropertyValue::Vec2(*value),
            AnimationChannelValueAsset::Vec3(value) => ScenePropertyValue::Vec3(*value),
            AnimationChannelValueAsset::Vec4(value) => ScenePropertyValue::Vec4(*value),
            AnimationChannelValueAsset::Quaternion(value) => ScenePropertyValue::Quaternion(*value),
        })
    }

    fn legacy_value_kind(value: &AnimationChannelValueAsset) -> &'static str {
        match value {
            AnimationChannelValueAsset::Bool(_) => "bool",
            AnimationChannelValueAsset::Integer(_) => "integer",
            AnimationChannelValueAsset::Scalar(_) => "scalar",
            AnimationChannelValueAsset::Vec2(_) => "vec2",
            AnimationChannelValueAsset::Vec3(_) => "vec3",
            AnimationChannelValueAsset::Vec4(_) => "vec4",
            AnimationChannelValueAsset::Quaternion(_) => "quaternion",
        }
    }

    fn legacy_value_is_finite(value: &AnimationChannelValueAsset) -> bool {
        match value {
            AnimationChannelValueAsset::Bool(_) | AnimationChannelValueAsset::Integer(_) => true,
            AnimationChannelValueAsset::Scalar(value) => value.is_finite(),
            AnimationChannelValueAsset::Vec2(value) => {
                value.iter().all(|component| component.is_finite())
            }
            AnimationChannelValueAsset::Vec3(value) => {
                value.iter().all(|component| component.is_finite())
            }
            AnimationChannelValueAsset::Vec4(value)
            | AnimationChannelValueAsset::Quaternion(value) => {
                value.iter().all(|component| component.is_finite())
            }
        }
    }

    fn measure(optimized: bool, values: &[AnimationChannelValueAsset], iterations: u64) -> u128 {
        let started = Instant::now();
        for index in 0..iterations {
            let value = black_box(&values[(index as usize) & 3]);
            let output = if optimized {
                scene_property_value_from_channel(value)
            } else {
                legacy_projection(value)
            };
            black_box(output.expect("finite benchmark value"));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
