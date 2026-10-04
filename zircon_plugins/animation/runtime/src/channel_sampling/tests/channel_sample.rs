use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn linear_interpolation_samples_midpoint_vec3_values() {
    let channel = AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: vec![
            key(0.0, AnimationChannelValueAsset::Vec3([0.0, 2.0, 4.0])),
            key(2.0, AnimationChannelValueAsset::Vec3([10.0, 6.0, 8.0])),
        ],
    };

    let sample = channel.sample(1.0).expect("midpoint sample");

    assert_eq!(sample, AnimationChannelValueAsset::Vec3([5.0, 4.0, 6.0]));
}

#[test]
fn linear_interpolation_slerps_quaternion_values() {
    let target = Quat::from_rotation_y(std::f32::consts::PI);
    let channel = AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: vec![
            key(
                0.0,
                AnimationChannelValueAsset::Quaternion(Quat::IDENTITY.to_array()),
            ),
            key(
                2.0,
                AnimationChannelValueAsset::Quaternion(target.to_array()),
            ),
        ],
    };

    let sample = channel.sample(1.0).expect("midpoint sample");
    let AnimationChannelValueAsset::Quaternion(value) = sample else {
        panic!("expected quaternion sample");
    };
    let midpoint = Quat::from_array(value);
    let expected = Quat::IDENTITY.slerp(target, 0.5).normalize();

    assert!((midpoint.length() - 1.0).abs() < 0.0001);
    assert!(midpoint.abs_diff_eq(expected, 0.0001));
}

#[test]
fn optimization_batch_20260830cf_channel_single_pass_still_rejects_late_non_finite_key() {
    let mut channel = scalar_channel(4_096);
    channel.keys[4_095].time_seconds = Real::NAN;

    assert_eq!(channel.sample(1.25), None);
}

#[test]
fn optimization_batch_20260830cf_channel_single_pass_preserves_step_boundary() {
    let mut channel = scalar_channel(3);
    channel.interpolation = AnimationInterpolationAsset::Step;

    assert_eq!(
        channel.sample(1.0),
        Some(AnimationChannelValueAsset::Scalar(0.0))
    );
    assert_eq!(
        channel.sample(1.000_1),
        Some(AnimationChannelValueAsset::Scalar(1.0))
    );
}

#[test]
fn optimization_batch_20260830cf_channel_single_pass_static_contract() {
    let source = include_str!("../channel_sample.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    let sample_start = production
        .find("    fn sample(&self, time_seconds: Real)")
        .expect("sample owner");
    let sample_end = production[sample_start..]
        .find("fn sample_linear(")
        .map(|offset| sample_start + offset)
        .expect("sample owner boundary");
    let sample = &production[sample_start..sample_end];

    assert!(sample.contains("let mut sampled_pair = None"));
    assert!(!sample.contains("self.keys.iter().any"));
    assert!(!sample.contains("self.keys.windows(2)"));
    assert_eq!(sample.matches("self.keys.iter()").count(), 1);
}

#[test]
#[ignore = "Release-only Runtime170 performance contract"]
fn optimization_batch_20260830cf_channel_single_pass_p95() {
    const KEY_COUNT: usize = 4_096;
    const ITERATIONS: usize = 2_000;
    const SAMPLES: usize = 17;
    let channel = scalar_channel(KEY_COUNT);
    let mut baseline_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        let baseline = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                assert!(!black_box(&channel)
                    .keys
                    .iter()
                    .any(|key| !key.time_seconds.is_finite()));
                black_box(channel.keys.windows(2).find(|pair| {
                    2_048.25 >= pair[0].time_seconds && 2_048.25 <= pair[1].time_seconds
                }));
            }
            started.elapsed().as_nanos()
        };
        let optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut sampled_pair = None;
                let mut previous = &channel.keys[0];
                for key in channel.keys.iter().skip(1) {
                    assert!(previous.time_seconds.is_finite());
                    if sampled_pair.is_none()
                        && 2_048.25 >= previous.time_seconds
                        && 2_048.25 <= key.time_seconds
                    {
                        sampled_pair = Some((previous, key));
                    }
                    previous = key;
                }
                assert!(previous.time_seconds.is_finite());
                black_box(sampled_pair);
            }
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
        "RUNTIME170_AUTHORED_CHANNEL_SINGLE_PASS_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(80),
        "expected single-pass validation and interval selection to reduce P95 by at least 20%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
    );
}

fn scalar_channel(key_count: usize) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: (0..key_count)
            .map(|index| {
                key(
                    index as Real,
                    AnimationChannelValueAsset::Scalar(index as Real),
                )
            })
            .collect(),
    }
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
}

fn key(time_seconds: Real, value: AnimationChannelValueAsset) -> AnimationChannelKeyAsset {
    AnimationChannelKeyAsset {
        time_seconds,
        value,
        in_tangent: None,
        out_tangent: None,
    }
}
