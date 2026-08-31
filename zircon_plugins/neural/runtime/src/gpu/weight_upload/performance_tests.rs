use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::NnWeightUploadPlan;

const BENCH_TENSOR_COUNT: usize = 4_096;
const BENCH_WEIGHT_BYTES: usize = 4 * 1024 * 1024;
const LOOKUP_CHECKS_PER_SAMPLE: usize = 64;
const CLONES_PER_SAMPLE: usize = 16;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn cloned_upload_plans_share_weight_bytes_and_dense_offsets() {
    let plan = benchmark_upload_plan();
    let cloned = plan.clone();

    assert!(Arc::ptr_eq(&plan.bytes, &cloned.bytes));
    assert!(Arc::ptr_eq(&plan.offsets, &cloned.offsets));
    assert_eq!(cloned.offset_for_tensor(2), Some(512));
    assert_eq!(cloned.offset_for_tensor(1), None);
    assert_eq!(cloned.offset_for_tensor(u16::MAX), None);
}

#[test]
#[ignore = "release-only shared weight upload bytes benchmark"]
fn shared_weight_upload_bytes_release_benchmark_evidence() {
    let legacy = vec![7_u8; BENCH_WEIGHT_BYTES];
    let optimized = Arc::<[u8]>::from(legacy.as_slice());

    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_legacy_clones(&legacy),
        || measure_shared_clones(&optimized),
    );
    report_and_gate(
        "shared_weight_upload_bytes",
        &legacy_samples,
        &optimized_samples,
        &format!(
            "weight_bytes={BENCH_WEIGHT_BYTES} clones_per_sample={CLONES_PER_SAMPLE} \
legacy_bytes_copied_per_sample={} optimized_bytes_copied_per_sample=0",
            BENCH_WEIGHT_BYTES * CLONES_PER_SAMPLE
        ),
        10,
        1,
        "shared weight bytes must reduce P95 by at least 90%",
    );
}

#[test]
#[ignore = "release-only dense weight offset lookup benchmark"]
fn dense_weight_offset_lookup_release_benchmark_evidence() {
    let plan = benchmark_upload_plan();
    let legacy = (0..BENCH_TENSOR_COUNT as u16)
        .step_by(2)
        .map(|tensor_id| (tensor_id, u64::from(tensor_id) * 256))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(legacy_offset_checksum(&legacy), offset_checksum(&plan));

    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_legacy_offsets(&legacy),
        || measure_dense_offsets(&plan),
    );
    report_and_gate(
        "dense_weight_offsets",
        &legacy_samples,
        &optimized_samples,
        &format!(
            "tensor_count={BENCH_TENSOR_COUNT} resident_weight_count={} \
checks_per_sample={LOOKUP_CHECKS_PER_SAMPLE} legacy_lookup=btree_log_n optimized_lookup=dense_o_1",
            BENCH_TENSOR_COUNT / 2
        ),
        2,
        1,
        "dense weight offsets must reduce P95 by at least 50%",
    );
}

fn benchmark_upload_plan() -> NnWeightUploadPlan {
    let offsets = (0..BENCH_TENSOR_COUNT)
        .map(|index| (index % 2 == 0).then_some(index as u64 * 256))
        .collect::<Vec<_>>();
    NnWeightUploadPlan {
        resource_name: "nn.weights".to_owned(),
        bytes: Arc::from(vec![0_u8; BENCH_WEIGHT_BYTES]),
        offsets: Arc::from(offsets),
    }
}

fn legacy_offset_checksum(offsets: &BTreeMap<u16, u64>) -> u64 {
    (0..BENCH_TENSOR_COUNT as u16)
        .filter_map(|tensor_id| offsets.get(&tensor_id))
        .copied()
        .sum()
}

fn offset_checksum(plan: &NnWeightUploadPlan) -> u64 {
    (0..BENCH_TENSOR_COUNT as u16)
        .filter_map(|tensor_id| plan.offset_for_tensor(tensor_id))
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

fn measure_legacy_clones(bytes: &[u8]) -> u128 {
    let started = Instant::now();
    for _ in 0..CLONES_PER_SAMPLE {
        black_box(bytes.to_vec());
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_shared_clones(bytes: &Arc<[u8]>) -> u128 {
    let started = Instant::now();
    for _ in 0..CLONES_PER_SAMPLE {
        black_box(Arc::clone(bytes));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_legacy_offsets(offsets: &BTreeMap<u16, u64>) -> u128 {
    let started = Instant::now();
    for _ in 0..LOOKUP_CHECKS_PER_SAMPLE {
        black_box(legacy_offset_checksum(black_box(offsets)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_dense_offsets(plan: &NnWeightUploadPlan) -> u128 {
    let started = Instant::now();
    for _ in 0..LOOKUP_CHECKS_PER_SAMPLE {
        black_box(offset_checksum(black_box(plan)));
    }
    started.elapsed().as_nanos().max(1)
}

fn report_and_gate(
    task: &str,
    legacy_samples: &[u128],
    optimized_samples: &[u128],
    workload: &str,
    maximum_ratio_denominator: u128,
    maximum_ratio_numerator: u128,
    message: &str,
) {
    let legacy_p50_ns = percentile(legacy_samples, 50);
    let optimized_p50_ns = percentile(optimized_samples, 50);
    let legacy_p95_ns = percentile(legacy_samples, 95);
    let optimized_p95_ns = percentile(optimized_samples, 95);
    println!(
        "PERF_RESULT plan=Plugins02 task={task} sample_pairs={SAMPLE_PAIRS} {workload} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(legacy_samples),
        raw(optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(maximum_ratio_denominator)
            <= legacy_p95_ns.saturating_mul(maximum_ratio_numerator),
        "{message}: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
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
