use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{InputBindings, TensorWorkspace};

const BENCH_TENSOR_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 21;
const WORKSPACE_CHECKS_PER_SAMPLE: usize = 8;
const INPUT_CHECKS_PER_SAMPLE: usize = 2;

#[test]
fn tensor_workspace_preserves_dense_slot_occupancy_and_take_semantics() {
    let mut workspace = TensorWorkspace::new(3);

    assert!(workspace.store(1, vec![2.0, 3.0]));
    assert_eq!(workspace.get(1), Some(&[2.0, 3.0][..]));
    assert_eq!(workspace.get(0), None);
    assert!(!workspace.store(3, vec![4.0]));
    assert_eq!(workspace.take(1), Some(vec![2.0, 3.0]));
    assert_eq!(workspace.get(1), None);
    assert_eq!(workspace.take(3), None);
}

#[test]
fn input_bindings_preserve_first_duplicate_and_ignore_out_of_range_ids() {
    let first = [2.0, 3.0];
    let duplicate = [7.0];
    let out_of_range = [11.0];
    let inputs = [
        (2, first.as_slice()),
        (2, duplicate.as_slice()),
        (9, out_of_range.as_slice()),
    ];

    let bindings = InputBindings::new(4, &inputs);

    assert_eq!(bindings.get(2), Some(first.as_slice()));
    assert_eq!(bindings.get(0), None);
    assert_eq!(bindings.get(9), None);
}

#[test]
#[ignore = "release-only dense tensor workspace benchmark"]
fn dense_tensor_workspace_release_benchmark_evidence() {
    assert_eq!(legacy_workspace_checksum(), optimized_workspace_checksum());

    let (legacy_samples, optimized_samples) =
        paired_samples(measure_legacy_workspace, measure_optimized_workspace);
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins02 task=dense_tensor_workspace \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={WORKSPACE_CHECKS_PER_SAMPLE} \
tensor_count={BENCH_TENSOR_COUNT} legacy_lookup=btree_log_n optimized_lookup=dense_o_1 \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "dense workspace must reduce P95 by at least 50%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
#[ignore = "release-only dense input binding lookup benchmark"]
fn dense_input_binding_lookup_release_benchmark_evidence() {
    let values = (0..BENCH_TENSOR_COUNT)
        .map(|index| [index as f32])
        .collect::<Vec<_>>();
    let inputs = values
        .iter()
        .enumerate()
        .rev()
        .map(|(index, value)| (index as u16, value.as_slice()))
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_input_checksum(&inputs),
        optimized_input_checksum(&inputs)
    );

    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_legacy_inputs(&inputs),
        || measure_optimized_inputs(&inputs),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins02 task=dense_input_bindings \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={INPUT_CHECKS_PER_SAMPLE} \
input_count={BENCH_TENSOR_COUNT} legacy_lookup=linear_per_model_input \
optimized_lookup=single_dense_projection pair_order=alternating_legacy_even \
legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns,
        "dense input bindings must reduce P95 by at least 80%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_workspace_checksum() -> usize {
    let mut tensors = BTreeMap::<u16, Vec<f32>>::new();
    for tensor_id in 0..BENCH_TENSOR_COUNT as u16 {
        tensors.insert(tensor_id, Vec::new());
    }
    let occupied = (0..BENCH_TENSOR_COUNT as u16)
        .filter(|tensor_id| tensors.get(tensor_id).is_some())
        .count();
    let removed = (0..BENCH_TENSOR_COUNT as u16)
        .filter(|tensor_id| tensors.remove(tensor_id).is_some())
        .count();
    occupied + removed
}

fn optimized_workspace_checksum() -> usize {
    let mut tensors = TensorWorkspace::new(BENCH_TENSOR_COUNT);
    for tensor_id in 0..BENCH_TENSOR_COUNT as u16 {
        assert!(tensors.store(tensor_id, Vec::new()));
    }
    let occupied = (0..BENCH_TENSOR_COUNT as u16)
        .filter(|tensor_id| tensors.get(*tensor_id).is_some())
        .count();
    let removed = (0..BENCH_TENSOR_COUNT as u16)
        .filter(|tensor_id| tensors.take(*tensor_id).is_some())
        .count();
    occupied + removed
}

fn legacy_input_checksum(inputs: &[(u16, &[f32])]) -> usize {
    (0..BENCH_TENSOR_COUNT as u16)
        .map(|tensor_id| {
            inputs
                .iter()
                .find(|(provided_id, _)| *provided_id == tensor_id)
                .map_or(0, |(_, values)| values.len())
        })
        .sum()
}

fn optimized_input_checksum(inputs: &[(u16, &[f32])]) -> usize {
    let bindings = InputBindings::new(BENCH_TENSOR_COUNT, inputs);
    (0..BENCH_TENSOR_COUNT as u16)
        .map(|tensor_id| bindings.get(tensor_id).map_or(0, <[f32]>::len))
        .sum()
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure_legacy_workspace() -> u128 {
    let started = Instant::now();
    for _ in 0..WORKSPACE_CHECKS_PER_SAMPLE {
        black_box(legacy_workspace_checksum());
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized_workspace() -> u128 {
    let started = Instant::now();
    for _ in 0..WORKSPACE_CHECKS_PER_SAMPLE {
        black_box(optimized_workspace_checksum());
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_legacy_inputs(inputs: &[(u16, &[f32])]) -> u128 {
    let started = Instant::now();
    for _ in 0..INPUT_CHECKS_PER_SAMPLE {
        black_box(legacy_input_checksum(black_box(inputs)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized_inputs(inputs: &[(u16, &[f32])]) -> u128 {
    let started = Instant::now();
    for _ in 0..INPUT_CHECKS_PER_SAMPLE {
        black_box(optimized_input_checksum(black_box(inputs)));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
