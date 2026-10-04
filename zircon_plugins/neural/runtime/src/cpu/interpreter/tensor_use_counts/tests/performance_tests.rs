use std::hint::black_box;
use std::mem::size_of;
use std::time::Instant;

use crate::{
    run_cpu, NnDataType, NnModelAsset, NnOp, NnOpAttrs, NnOpCode, NnTensorDesc, NnTensorKind,
};

use super::super::apply_unary_value;
use super::TensorUseCounts;

const BENCH_ELEMENTS: usize = 1_048_576;
const SAMPLE_PAIRS: usize = 21;
const UNARY_CHAIN_LENGTH: usize = 8;

#[test]
fn use_counts_preserve_branched_inputs_until_the_last_consumer() {
    let model = branched_unary_model();
    let mut uses = TensorUseCounts::new(&model);

    assert!(uses.is_last_consumer(0));
    assert!(!uses.is_last_consumer(1));
    assert_eq!(uses.remaining(1), 2);
    assert_eq!(uses.remaining(2), 1);
    assert_eq!(uses.remaining(3), 1);

    uses.consume(&[0]);
    uses.consume(&[1]);
    assert!(uses.is_last_consumer(1));
    uses.consume(&[1]);
    assert_eq!(uses.remaining(1), 0);
}

#[test]
fn last_use_unary_reuse_preserves_branched_output_values() {
    let model = branched_unary_model();
    let outputs = run_cpu(&model, &[(0, &[-1.0, 2.0])]).unwrap();

    assert_eq!(
        outputs,
        vec![
            vec![0.0, 2.0_f32.tanh()],
            vec![0.5, 1.0 / (1.0 + (-2.0_f32).exp())],
        ]
    );
}

#[test]
#[ignore = "release-only last-use unary buffer reuse benchmark"]
fn last_use_unary_buffer_reuse_release_benchmark_evidence() {
    let seed = (0..BENCH_ELEMENTS)
        .map(|index| index as f32 % 257.0 - 128.0)
        .collect::<Vec<_>>();
    assert_eq!(legacy_chain(&seed), optimized_chain(&seed));

    let (legacy_samples, optimized_samples) =
        paired_samples(|| measure_legacy(&seed), || measure_optimized(&seed));
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins02 task=last_use_unary_buffer_reuse \
sample_pairs={SAMPLE_PAIRS} elements={BENCH_ELEMENTS} unary_chain_length={UNARY_CHAIN_LENGTH} \
legacy_output_allocations_per_sample={UNARY_CHAIN_LENGTH} optimized_output_allocations_per_sample=0 \
legacy_output_bytes_allocated_per_sample={} optimized_output_bytes_allocated_per_sample=0 \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        BENCH_ELEMENTS * size_of::<f32>() * UNARY_CHAIN_LENGTH,
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(4),
        "last-use unary reuse must reduce P95 by at least 20%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn branched_unary_model() -> NnModelAsset {
    NnModelAsset {
        tensors: vec![
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Input, 4, [1, 1, 1, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Intermediate, 4, [1, 1, 1, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 4, [1, 1, 1, 2]),
            NnTensorDesc::new(NnDataType::F32, NnTensorKind::Output, 4, [1, 1, 1, 2]),
        ],
        ops: vec![
            NnOp::new(NnOpCode::Relu, vec![0], vec![1], NnOpAttrs::None),
            NnOp::new(NnOpCode::Tanh, vec![1], vec![2], NnOpAttrs::None),
            NnOp::new(NnOpCode::Sigmoid, vec![1], vec![3], NnOpAttrs::None),
        ],
        weights: Vec::new(),
    }
}

fn legacy_chain(seed: &[f32]) -> Vec<f32> {
    let mut values = seed.to_vec();
    for _ in 0..UNARY_CHAIN_LENGTH {
        values = values
            .iter()
            .map(|value| apply_unary_value(NnOpCode::Relu, *value))
            .collect();
    }
    values
}

fn optimized_chain(seed: &[f32]) -> Vec<f32> {
    let mut values = seed.to_vec();
    for _ in 0..UNARY_CHAIN_LENGTH {
        for value in &mut values {
            *value = apply_unary_value(NnOpCode::Relu, *value);
        }
    }
    values
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

fn measure_legacy(seed: &[f32]) -> u128 {
    measure(|| legacy_chain(black_box(seed)))
}

fn measure_optimized(seed: &[f32]) -> u128 {
    measure(|| optimized_chain(black_box(seed)))
}

fn measure(run: impl FnOnce() -> Vec<f32>) -> u128 {
    let started = Instant::now();
    let output = run();
    black_box(&output);
    let elapsed = started.elapsed().as_nanos().max(1);
    black_box(output);
    elapsed
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
