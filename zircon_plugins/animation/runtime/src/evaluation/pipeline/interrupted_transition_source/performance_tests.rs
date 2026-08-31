use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::framework::animation::{AnimationPoseOutput, AnimationPoseSource};

use super::{replace_interrupted_transition_source, InterruptedTransitionSource};

const STATE_BYTES: usize = 256;
const UPDATES_PER_SAMPLE: usize = 8_192;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

#[test]
fn interrupted_transition_replacement_reuses_strings_and_replaces_pose() {
    let original_pose = Arc::new(pose("Old"));
    let replacement_pose = Arc::new(pose("New"));
    let mut source = InterruptedTransitionSource {
        from_state: "from-state-with-retained-capacity".to_string(),
        to_state: "to-state-with-retained-capacity".to_string(),
        pose: Arc::clone(&original_pose),
    };
    let from_capacity = source.from_state.capacity();
    let to_capacity = source.to_state.capacity();

    replace_interrupted_transition_source(&mut source, "from", "to", Arc::clone(&replacement_pose));

    assert_eq!(source.from_state, "from");
    assert_eq!(source.to_state, "to");
    assert!(source.from_state.capacity() >= from_capacity);
    assert!(source.to_state.capacity() >= to_capacity);
    assert!(Arc::ptr_eq(&source.pose, &replacement_pose));
}

#[test]
#[ignore = "release-only performance gate"]
fn interrupted_transition_string_reuse_release_benchmark_evidence() {
    let from_state = "f".repeat(STATE_BYTES);
    let to_state = "t".repeat(STATE_BYTES);
    let pose = Arc::new(pose("Benchmark"));
    let (legacy_samples, optimized_samples) = paired_samples(
        || legacy_replacements(&from_state, &to_state, &pose),
        || optimized_replacements(&from_state, &to_state, &pose),
    );
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);

    println!(
        "PERF_RESULT task=runtime170_interrupted_transition_string_reuse state_bytes={STATE_BYTES} updates_per_sample={UPDATES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_string_allocations_per_sample={} optimized_string_allocations_per_sample=2 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT} legacy_raw_ns={} optimized_raw_ns={}",
        UPDATES_PER_SAMPLE * 2,
        samples_csv(&legacy_samples),
        samples_csv(&optimized_samples),
    );
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "interrupted transition string reuse must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn pose(active_state: &str) -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::StateMachine,
        active_state: Some(active_state.to_string()),
        bones: Vec::new(),
    }
}

fn legacy_replacements(
    from_state: &str,
    to_state: &str,
    pose: &Arc<AnimationPoseOutput>,
) -> InterruptedTransitionSource {
    let mut source = InterruptedTransitionSource {
        from_state: String::new(),
        to_state: String::new(),
        pose: Arc::clone(pose),
    };
    for _ in 0..UPDATES_PER_SAMPLE {
        source = InterruptedTransitionSource {
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            pose: Arc::clone(pose),
        };
        black_box(&source);
    }
    source
}

fn optimized_replacements(
    from_state: &str,
    to_state: &str,
    pose: &Arc<AnimationPoseOutput>,
) -> InterruptedTransitionSource {
    let mut source = InterruptedTransitionSource {
        from_state: String::new(),
        to_state: String::new(),
        pose: Arc::clone(pose),
    };
    for _ in 0..UPDATES_PER_SAMPLE {
        replace_interrupted_transition_source(&mut source, from_state, to_state, Arc::clone(pose));
        black_box(&source);
    }
    source
}

fn paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
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
