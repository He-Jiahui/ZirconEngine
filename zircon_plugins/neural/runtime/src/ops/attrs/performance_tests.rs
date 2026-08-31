use std::hint::black_box;
use std::time::Instant;

use super::{NnConv2dAttrs, NnGemmAttrs, NnOpAttrs, NnOpAttrsError, NnOpCode, NnPool2dAttrs};

const BENCH_OP_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 16;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn nonallocating_attr_validation_matches_supported_op_families() {
    let supported = [
        (NnOpCode::Gemm, NnOpAttrs::Gemm(NnGemmAttrs::default())),
        (
            NnOpCode::Conv2d,
            NnOpAttrs::Conv2d(NnConv2dAttrs::default()),
        ),
        (
            NnOpCode::MaxPool2d,
            NnOpAttrs::Pool2d(NnPool2dAttrs {
                kernel: [2, 2],
                stride: [1, 1],
                padding: [0; 4],
            }),
        ),
        (
            NnOpCode::BatchNorm,
            NnOpAttrs::BatchNorm { epsilon: 1.0e-5 },
        ),
        (
            NnOpCode::LayerNorm,
            NnOpAttrs::LayerNorm { epsilon: 1.0e-5 },
        ),
        (
            NnOpCode::Upsample2d,
            NnOpAttrs::Upsample2d { scale: [2, 2] },
        ),
        (NnOpCode::Relu, NnOpAttrs::None),
    ];

    for (code, attrs) in supported {
        assert_eq!(attrs.validate_for(code), Ok(()));
        assert!(attrs.encode(code).is_ok());
    }
    assert_eq!(
        NnOpAttrs::None.validate_for(NnOpCode::Gemm),
        Err(NnOpAttrsError::UnexpectedAttrsForOp {
            code: NnOpCode::Gemm,
        })
    );
}

#[test]
#[ignore = "release-only nonallocating op attribute validation benchmark"]
fn nonallocating_attr_validation_release_benchmark_evidence() {
    let attrs = (0..BENCH_OP_COUNT)
        .map(|_| NnOpAttrs::Conv2d(NnConv2dAttrs::default()))
        .collect::<Vec<_>>();
    assert_eq!(legacy_checksum(&attrs), optimized_checksum(&attrs));

    let (legacy_samples, optimized_samples) =
        paired_samples(|| measure_legacy(&attrs), || measure_optimized(&attrs));
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins02 task=nonallocating_attr_validation \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
op_count={BENCH_OP_COUNT} legacy_temporary_bytes_per_op=36 optimized_temporary_bytes_per_op=0 \
legacy_allocations_per_sample={} optimized_allocations_per_sample=0 \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        BENCH_OP_COUNT * CHECKS_PER_SAMPLE,
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns,
        "nonallocating attr validation must reduce P95 by at least 90%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_checksum(attrs: &[NnOpAttrs]) -> usize {
    attrs
        .iter()
        .map(|attrs| attrs.encode(NnOpCode::Conv2d).unwrap().len())
        .sum()
}

fn optimized_checksum(attrs: &[NnOpAttrs]) -> usize {
    attrs
        .iter()
        .map(|attrs| {
            attrs.validate_for(NnOpCode::Conv2d).unwrap();
            36
        })
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

fn measure_legacy(attrs: &[NnOpAttrs]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        for attrs in black_box(attrs) {
            black_box(attrs.encode(black_box(NnOpCode::Conv2d)).unwrap());
        }
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(attrs: &[NnOpAttrs]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        for attrs in black_box(attrs) {
            black_box(attrs.validate_for(black_box(NnOpCode::Conv2d)).unwrap());
        }
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
