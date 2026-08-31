use crate::core::framework::animation::{AnimationChannelKeyAsset, AnimationChannelValueAsset};
use crate::core::math::{Quat, Real};

pub(super) fn sample_hermite(
    left: &AnimationChannelKeyAsset,
    right: &AnimationChannelKeyAsset,
    time_seconds: Real,
) -> AnimationChannelValueAsset {
    let duration = (right.time_seconds - left.time_seconds).max(Real::EPSILON);
    let t = ((time_seconds - left.time_seconds) / duration).clamp(0.0, 1.0);

    match (&left.value, &right.value) {
        (
            AnimationChannelValueAsset::Scalar(left_value),
            AnimationChannelValueAsset::Scalar(right_value),
        ) => {
            let left_tangent = tangent_scalar(left.out_tangent.as_ref());
            let right_tangent = tangent_scalar(right.in_tangent.as_ref());
            AnimationChannelValueAsset::Scalar(hermite_scalar(
                *left_value,
                left_tangent,
                *right_value,
                right_tangent,
                duration,
                t,
            ))
        }
        (
            AnimationChannelValueAsset::Vec2(left_value),
            AnimationChannelValueAsset::Vec2(right_value),
        ) => AnimationChannelValueAsset::Vec2(hermite_array(
            left_value,
            tangent_array_2(left.out_tangent.as_ref()),
            right_value,
            tangent_array_2(right.in_tangent.as_ref()),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Vec3(left_value),
            AnimationChannelValueAsset::Vec3(right_value),
        ) => AnimationChannelValueAsset::Vec3(hermite_array(
            left_value,
            tangent_array_3(left.out_tangent.as_ref()),
            right_value,
            tangent_array_3(right.in_tangent.as_ref()),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Vec4(left_value),
            AnimationChannelValueAsset::Vec4(right_value),
        ) => AnimationChannelValueAsset::Vec4(hermite_array(
            left_value,
            tangent_array_4(left.out_tangent.as_ref()),
            right_value,
            tangent_array_4(right.in_tangent.as_ref()),
            duration,
            t,
        )),
        (
            AnimationChannelValueAsset::Quaternion(left_value),
            AnimationChannelValueAsset::Quaternion(right_value),
        ) => {
            let left_quat = Quat::from_array(*left_value).normalize();
            let right_quat = Quat::from_array(*right_value).normalize();
            AnimationChannelValueAsset::Quaternion(
                left_quat.slerp(right_quat, t).normalize().to_array(),
            )
        }
        _ => left.value.clone(),
    }
}

fn hermite_scalar(
    left_value: Real,
    left_tangent: Real,
    right_value: Real,
    right_tangent: Real,
    duration: Real,
    t: Real,
) -> Real {
    HermiteBasis::new(t).sample(
        left_value,
        left_tangent,
        right_value,
        right_tangent,
        duration,
    )
}

#[derive(Clone, Copy)]
struct HermiteBasis {
    h00: Real,
    h10: Real,
    h01: Real,
    h11: Real,
}

impl HermiteBasis {
    fn new(t: Real) -> Self {
        let t2 = t * t;
        let t3 = t2 * t;
        Self {
            h00: 2.0 * t3 - 3.0 * t2 + 1.0,
            h10: t3 - 2.0 * t2 + t,
            h01: -2.0 * t3 + 3.0 * t2,
            h11: t3 - t2,
        }
    }

    fn sample(
        self,
        left_value: Real,
        left_tangent: Real,
        right_value: Real,
        right_tangent: Real,
        duration: Real,
    ) -> Real {
        self.h00 * left_value
            + self.h10 * left_tangent * duration
            + self.h01 * right_value
            + self.h11 * right_tangent * duration
    }
}

fn hermite_array<const N: usize>(
    left_value: &[Real; N],
    left_tangent: [Real; N],
    right_value: &[Real; N],
    right_tangent: [Real; N],
    duration: Real,
    t: Real,
) -> [Real; N] {
    let basis = HermiteBasis::new(t);
    let mut result = [0.0; N];
    let mut index = 0;
    while index < N {
        result[index] = basis.sample(
            left_value[index],
            left_tangent[index],
            right_value[index],
            right_tangent[index],
            duration,
        );
        index += 1;
    }
    result
}

fn tangent_scalar(value: Option<&AnimationChannelValueAsset>) -> Real {
    match value {
        Some(AnimationChannelValueAsset::Scalar(value)) => *value,
        _ => 0.0,
    }
}

fn tangent_array_2(value: Option<&AnimationChannelValueAsset>) -> [Real; 2] {
    match value {
        Some(AnimationChannelValueAsset::Vec2(value)) => *value,
        _ => [0.0; 2],
    }
}

fn tangent_array_3(value: Option<&AnimationChannelValueAsset>) -> [Real; 3] {
    match value {
        Some(AnimationChannelValueAsset::Vec3(value)) => *value,
        _ => [0.0; 3],
    }
}

fn tangent_array_4(value: Option<&AnimationChannelValueAsset>) -> [Real; 4] {
    match value {
        Some(AnimationChannelValueAsset::Vec4(value)) => *value,
        Some(AnimationChannelValueAsset::Quaternion(value)) => *value,
        _ => [0.0; 4],
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const LEFT: [Real; 4] = [1.0, 2.0, 3.0, 4.0];
    const LEFT_TANGENT: [Real; 4] = [0.2, 0.3, 0.4, 0.5];
    const RIGHT: [Real; 4] = [5.0, 6.0, 7.0, 8.0];
    const RIGHT_TANGENT: [Real; 4] = [0.6, 0.7, 0.8, 0.9];

    #[test]
    fn optimization_batch_20260831fb_runtime567_shared_basis_preserves_vec4_samples() {
        for t in [0.0, 0.1, 0.5, 0.9, 1.0] {
            assert_eq!(
                hermite_array(&LEFT, LEFT_TANGENT, &RIGHT, RIGHT_TANGENT, 2.5, t),
                legacy_hermite_array(&LEFT, LEFT_TANGENT, &RIGHT, RIGHT_TANGENT, 2.5, t)
            );
        }
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260831fb_runtime567_shared_basis_vec4_p95() {
        const SAMPLE_PAIRS: usize = 13;
        const ITERATIONS: u64 = 10_000_000;
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, ITERATIONS));
                optimized.push(measure(true, ITERATIONS));
            } else {
                optimized.push(measure(true, ITERATIONS));
                legacy.push(measure(false, ITERATIONS));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME567_HERMITE_BASIS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85));
    }

    fn legacy_hermite_array<const N: usize>(
        left: &[Real; N],
        left_tangent: [Real; N],
        right: &[Real; N],
        right_tangent: [Real; N],
        duration: Real,
        t: Real,
    ) -> [Real; N] {
        let mut result = [0.0; N];
        let mut index = 0;
        while index < N {
            let t2 = t * t;
            let t3 = t2 * t;
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;
            result[index] = h00 * left[index]
                + h10 * left_tangent[index] * duration
                + h01 * right[index]
                + h11 * right_tangent[index] * duration;
            index += 1;
        }
        result
    }

    fn measure(optimized: bool, iterations: u64) -> u128 {
        let started = Instant::now();
        let mut checksum = 0.0;
        for index in 0..iterations {
            let t = black_box(((index & 1023) as Real + 0.5) / 1024.0);
            let values = if optimized {
                hermite_array(&LEFT, LEFT_TANGENT, &RIGHT, RIGHT_TANGENT, 2.5, t)
            } else {
                legacy_hermite_array(&LEFT, LEFT_TANGENT, &RIGHT, RIGHT_TANGENT, 2.5, t)
            };
            checksum += values[(index as usize) & 3];
        }
        black_box(checksum);
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
