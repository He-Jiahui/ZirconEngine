use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::export::ExportStage;

use super::ExportPipelinePlanError;

const SAMPLE_PAIRS: usize = 21;
const FORMATS_PER_SAMPLE: usize = 16_384;
const STAGES_PER_CYCLE: usize = 32;

#[test]
fn optimization_batch_20260826dn_editor103_export_cycle_format_preserves_cli_ids() {
    let error = ExportPipelinePlanError::DependencyCycle {
        stages: vec![
            ExportStage::Validate,
            ExportStage::CookAssets,
            ExportStage::PlatformBundle,
        ],
    };
    assert_eq!(
        error.to_string(),
        "export stage dependency cycle contains: validate, cook_assets, platform_bundle"
    );
}

#[test]
fn optimization_batch_20260826dn_editor103_export_cycle_format_writes_formatter_directly() {
    let source = include_str!("../pipeline.rs");
    assert!(source.contains("write_dependency_cycle(formatter, stages)"));
    assert!(source.contains("formatter.write_str(stage.cli_id())?;"));
    assert!(
        !source.contains(".map(|stage| stage.cli_id())\n                    .collect::<Vec<_>>()")
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dn_editor103_export_cycle_direct_format_bench() {
    let stages = fixture_stages();
    let error = ExportPipelinePlanError::DependencyCycle {
        stages: stages.clone(),
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(|| legacy_format(&stages)));
            optimized_samples.push(measure(|| error.to_string()));
        } else {
            optimized_samples.push(measure(|| error.to_string()));
            legacy_samples.push(measure(|| legacy_format(&stages)));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR103_EXPORT_CYCLE_DIRECT_FORMAT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
formats_per_sample={FORMATS_PER_SAMPLE} stages_per_cycle={STAGES_PER_CYCLE} \
legacy_intermediate_allocations_per_format=2 optimized_intermediate_allocations_per_format=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct dependency-cycle formatting P95 {optimized_p95_ns}ns must be at most 70% of collected CLI-id formatting P95 {legacy_p95_ns}ns"
    );
}

fn fixture_stages() -> Vec<ExportStage> {
    ExportStage::ALL
        .into_iter()
        .cycle()
        .take(STAGES_PER_CYCLE)
        .collect()
}

fn legacy_format(stages: &[ExportStage]) -> String {
    format!(
        "export stage dependency cycle contains: {}",
        stages
            .iter()
            .map(|stage| stage.cli_id())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn measure(mut render: impl FnMut() -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..FORMATS_PER_SAMPLE {
        checksum ^= black_box(render()).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
