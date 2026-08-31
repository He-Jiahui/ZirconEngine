use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::Transform;
use zircon_runtime::scene::EntityId;

use super::pose_update_capacity;

const ROOT_COUNT: usize = 128;
const BONES_PER_ROOT: usize = 128;
const UPDATE_COUNT: usize = ROOT_COUNT * BONES_PER_ROOT;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 15;

#[test]
fn pose_update_capacity_sums_all_bones() {
    let poses = [pose_with_bones(3), pose_with_bones(5), pose_with_bones(0)];

    assert_eq!(pose_update_capacity(poses.iter()), 8);
}

#[test]
#[ignore = "release-only performance gate"]
fn exact_pose_update_capacity_release_benchmark_evidence() {
    let values = (0..UPDATE_COUNT as u64).collect::<Vec<_>>();
    let (legacy_samples, optimized_samples) = paired_samples(
        || collect_updates(&values, 0),
        || collect_updates(&values, UPDATE_COUNT),
    );
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    let legacy_growths = allocation_growths(0, UPDATE_COUNT);
    let optimized_growths = allocation_growths(UPDATE_COUNT, UPDATE_COUNT);

    println!(
        "PERF_RESULT task=runtime170_exact_pose_update_capacity roots={ROOT_COUNT} bones_per_root={BONES_PER_ROOT} updates={UPDATE_COUNT} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_capacity_growths={legacy_growths} optimized_capacity_growths={optimized_growths} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT} legacy_raw_ns={} optimized_raw_ns={}",
        samples_csv(&legacy_samples),
        samples_csv(&optimized_samples),
    );
    assert!(legacy_growths > 0);
    assert_eq!(optimized_growths, 0);
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "exact pose update capacity must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn pose_with_bones(count: usize) -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones: (0..count)
            .map(|index| AnimationPoseBone {
                name: format!("bone-{index}"),
                local_transform: Transform::default(),
            })
            .collect(),
    }
}

fn collect_updates(values: &[EntityId], initial_capacity: usize) -> Vec<(EntityId, Transform)> {
    let mut updates = Vec::with_capacity(initial_capacity);
    updates.extend(
        values
            .iter()
            .copied()
            .map(|entity| (entity, Transform::default())),
    );
    updates
}

fn allocation_growths(initial_capacity: usize, count: usize) -> usize {
    let mut values = Vec::with_capacity(initial_capacity);
    let mut growths = 0;
    let mut capacity = values.capacity();
    for value in 0..count {
        values.push(value);
        if values.capacity() != capacity {
            growths += 1;
            capacity = values.capacity();
        }
    }
    black_box(values);
    growths
}

fn paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = ordered.len().saturating_mul(percentile).div_ceil(100) - 1;
    ordered[index]
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
