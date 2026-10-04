use std::cell::Cell;
use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::animation::compiler::sequence::{
    compile_animation_sequence, AnimationCompiledSequenceTrack,
};
use crate::core::framework::animation::{
    AnimationSequenceAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
};
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};

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
fn step_interpolation_keeps_the_preceding_value_at_an_exact_interior_key() {
    let channel = AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![
            key(0.0, AnimationChannelValueAsset::Scalar(1.0)),
            key(1.0, AnimationChannelValueAsset::Scalar(2.0)),
            key(2.0, AnimationChannelValueAsset::Scalar(3.0)),
        ],
    };

    assert_eq!(
        channel.sample(1.0),
        Some(AnimationChannelValueAsset::Scalar(1.0))
    );
    assert_eq!(
        channel.sample(1.000_1),
        Some(AnimationChannelValueAsset::Scalar(2.0))
    );
}

#[test]
fn raw_channel_sampling_rejects_a_non_finite_key_time() {
    let channel = AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: vec![
            key(0.0, AnimationChannelValueAsset::Scalar(1.0)),
            key(Real::NAN, AnimationChannelValueAsset::Scalar(2.0)),
        ],
    };

    assert_eq!(channel.sample(0.5), None);
}

#[test]
fn compiled_step_sampling_preserves_exact_interior_key_hold_behavior() {
    let track = compiled_track(
        AnimationInterpolationAsset::Step,
        vec![
            key(0.0, AnimationChannelValueAsset::Scalar(1.0)),
            key(1.0, AnimationChannelValueAsset::Scalar(2.0)),
            key(2.0, AnimationChannelValueAsset::Scalar(3.0)),
        ],
    );

    assert_eq!(
        track.sample_compiled(1.0),
        Some(AnimationChannelValueAsset::Scalar(1.0))
    );
    assert_eq!(
        track.sample_compiled(1.000_1),
        Some(AnimationChannelValueAsset::Scalar(2.0))
    );
    assert_eq!(track.sample_compiled(Real::NAN), None);
}

#[test]
fn compiled_sampling_source_does_not_rescan_validated_key_times() {
    let production = include_str!("../channel_sample.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production channel sampler source");
    let compiled_impl = production
        .split("impl AnimationCompiledSequenceTrackSampleExt")
        .nth(1)
        .and_then(|source| source.split("trait AnimationChannelKeyView").next())
        .expect("compiled sequence track sampler implementation");

    assert!(compiled_impl.contains("sample_channel_keys("));
    assert!(!compiled_impl.contains(".iter().any"));
    assert!(production.contains("keys.partition_point"));
}

#[test]
fn compiled_sampling_interval_lookup_has_logarithmic_key_time_work() {
    const KEY_COUNT: usize = 16_384;
    let visits = Cell::new(0usize);
    let keys = (0..KEY_COUNT)
        .map(|index| CountingKey {
            time_seconds: index as Real,
            value: AnimationChannelValueAsset::Scalar(index as Real),
            visits: &visits,
        })
        .collect::<Vec<_>>();

    let sample = sample_channel_keys(
        AnimationInterpolationAsset::Linear,
        &keys,
        KEY_COUNT as Real / 2.0 + 0.5,
    );

    assert_eq!(
        sample,
        Some(AnimationChannelValueAsset::Scalar(
            KEY_COUNT as Real / 2.0 + 0.5
        ))
    );
    assert!(
        visits.get() <= KEY_COUNT.ilog2() as usize + 8,
        "compiled interval lookup should visit logarithmically many key times; visited {} for {KEY_COUNT} keys",
        visits.get()
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn compiled_channel_sampling_long_track_p95() {
    const KEY_COUNT: usize = 16_384;
    const ITERATIONS: usize = 256;
    const SAMPLE_COUNT: usize = 101;
    let channel = AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Linear,
        keys: (0..KEY_COUNT)
            .map(|index| {
                key(
                    index as Real * 0.001,
                    AnimationChannelValueAsset::Scalar(index as Real),
                )
            })
            .collect(),
    };
    let track = compiled_track(channel.interpolation, channel.keys.clone());
    let sample_time = KEY_COUNT as Real * 0.000_5;
    let mut raw_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut compiled_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample_index in 0..SAMPLE_COUNT {
        let measure_raw = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(channel.sample(black_box(sample_time)));
            }
            started.elapsed().as_nanos()
        };
        let measure_compiled = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(track.sample_compiled(black_box(sample_time)));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            raw_samples.push(measure_raw());
            compiled_samples.push(measure_compiled());
        } else {
            compiled_samples.push(measure_compiled());
            raw_samples.push(measure_raw());
        }
    }

    raw_samples.sort_unstable();
    compiled_samples.sort_unstable();
    let p50 = percentile_index(SAMPLE_COUNT, 50);
    let p95 = percentile_index(SAMPLE_COUNT, 95);
    println!(
        "RUNTIME170_COMPILED_CHANNEL_SAMPLE_V1 keys={} iterations={} samples={} sample_count={} raw_p50_ns={} raw_p95_ns={} compiled_p50_ns={} compiled_p95_ns={} raw_samples={} compiled_samples={}",
        KEY_COUNT,
        ITERATIONS,
        SAMPLE_COUNT,
        SAMPLE_COUNT,
        raw_samples[p50],
        raw_samples[p95],
        compiled_samples[p50],
        compiled_samples[p95],
        format_samples(&raw_samples),
        format_samples(&compiled_samples),
    );
    assert!(compiled_samples[p95].saturating_mul(4) < raw_samples[p95]);
}

fn percentile_index(sample_count: usize, percentile: usize) -> usize {
    sample_count
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1)
}

fn format_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn compiled_track(
    interpolation: AnimationInterpolationAsset,
    keys: Vec<AnimationChannelKeyAsset>,
) -> AnimationCompiledSequenceTrack {
    let sequence = AnimationSequenceAsset {
        name: Some("Compiled sampler test".to_string()),
        duration_seconds: keys
            .last()
            .map(|key| key.time_seconds.max(0.0))
            .unwrap_or_default(),
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation.x").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation,
                    keys,
                },
            }],
        }],
    };

    compile_animation_sequence(&sequence)
        .artifact()
        .expect("valid test sequence should compile")
        .bindings()[0]
        .tracks()[0]
        .clone()
}

struct CountingKey<'a> {
    time_seconds: Real,
    value: AnimationChannelValueAsset,
    visits: &'a Cell<usize>,
}

impl AnimationChannelKeyView for CountingKey<'_> {
    fn time_seconds(&self) -> Real {
        self.visits.set(self.visits.get() + 1);
        self.time_seconds
    }

    fn value(&self) -> &AnimationChannelValueAsset {
        &self.value
    }

    fn in_tangent(&self) -> Option<&AnimationChannelValueAsset> {
        None
    }

    fn out_tangent(&self) -> Option<&AnimationChannelValueAsset> {
        None
    }
}

fn key(time_seconds: Real, value: AnimationChannelValueAsset) -> AnimationChannelKeyAsset {
    AnimationChannelKeyAsset {
        time_seconds,
        value,
        in_tangent: None,
        out_tangent: None,
    }
}
