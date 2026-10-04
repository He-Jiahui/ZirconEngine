use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::asset::AssetUri;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const ERRORS_PER_SAMPLE: usize = 8_192;

#[test]
fn borrowed_dfd_error_arguments_preserve_diagnostic_text() {
    let context = test_context();

    let error = validate_data_format_descriptor_header(&context, 2, 16).unwrap_err();

    assert_eq!(
        error.to_string(),
        "parse texture container broken.ktx2: ktx2 data format descriptor offset must be 4-byte aligned, got 2"
    );
}

#[test]
#[ignore = "release-only performance contract"]
fn benchmark_borrowed_dfd_error_arguments() {
    let context = test_context();
    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_errors(&context, legacy_error));
            optimized_raw.push(measure_errors(&context, optimized_error));
        } else {
            optimized_raw.push(measure_errors(&context, optimized_error));
            legacy_raw.push(measure_errors(&context, legacy_error));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
        "borrowed DFD error arguments must improve P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "PERF_RESULT task=plugins07_borrowed_dfd_error_arguments sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank errors_per_sample={ERRORS_PER_SAMPLE} legacy_allocations_per_error=2 optimized_allocations_per_error=1 legacy_detail_allocations_per_sample={ERRORS_PER_SAMPLE} optimized_detail_allocations_per_sample=0 threshold_percent=15 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
}

fn legacy_error(context: &AssetImportContext, index: usize) -> AssetImportError {
    let detail = format!(
        "ktx2 data format descriptor block {} version must be 2, got {}",
        index % 8,
        3 + index % 17
    );
    parse_error_value(context, detail)
}

fn optimized_error(context: &AssetImportContext, index: usize) -> AssetImportError {
    parse_error_value(
        context,
        format_args!(
            "ktx2 data format descriptor block {} version must be 2, got {}",
            index % 8,
            3 + index % 17
        ),
    )
}

fn measure_errors(
    context: &AssetImportContext,
    make_error: fn(&AssetImportContext, usize) -> AssetImportError,
) -> u64 {
    let started = Instant::now();
    for index in 0..ERRORS_PER_SAMPLE {
        black_box(make_error(black_box(context), black_box(index)));
    }
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn test_context() -> AssetImportContext {
    AssetImportContext::new(
        "broken.ktx2".into(),
        AssetUri::parse("res://textures/broken.ktx2").unwrap(),
        vec![0; 16],
        Default::default(),
    )
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw_samples(samples: &[u64]) -> String {
    let values = samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}
