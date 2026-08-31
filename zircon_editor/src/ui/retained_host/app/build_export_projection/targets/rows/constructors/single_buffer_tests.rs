use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::project::ExportPackagingStrategy;

use super::{diagnostic_summary, strategy_summary};

const SAMPLE_PAIRS: usize = 21;
const SUMMARIES_PER_SAMPLE: usize = 8_192;
const STRATEGIES_PER_SUMMARY: usize = 32;

#[test]
fn optimization_batch_20260826dj_editor99_export_row_summaries_preserve_output() {
    use ExportPackagingStrategy::{LibraryEmbed, NativeDynamic, SourceTemplate};

    assert_eq!(strategy_summary(&[]), "");
    assert_eq!(
        strategy_summary(&[SourceTemplate, LibraryEmbed, NativeDynamic]),
        "SourceTemplate, LibraryEmbed, NativeDynamic"
    );
    assert_eq!(
        diagnostic_summary(
            &["fatal one".to_string(), "fatal two".to_string()],
            &["warning".to_string()]
        ),
        "fatal one\nfatal two\nwarning"
    );
    assert_eq!(diagnostic_summary(&[], &[]), "");
}

#[test]
fn optimization_batch_20260826dj_editor99_export_row_summaries_use_single_buffers() {
    let strategies = fixture_strategies();
    let strategy_text = strategy_summary(&strategies);
    assert_eq!(strategy_text.len(), strategy_text.capacity());

    let fatal = vec!["fatal".to_string(); 4];
    let diagnostics = vec!["warning".to_string(); 4];
    let diagnostic_text = diagnostic_summary(&fatal, &diagnostics);
    assert_eq!(diagnostic_text.len(), diagnostic_text.capacity());

    let source = include_str!("../constructors.rs");
    assert!(source.contains("diagnostic_summary(&plan.fatal_diagnostics, &plan.diagnostics)"));
    assert_eq!(source.matches("strategy_summary(").count(), 3);
    assert!(!source.contains("collect::<Vec<_>>()"));
    assert!(!source.contains("format!(\"{strategy:?}\")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dj_editor99_export_row_single_buffer_summaries_bench() {
    let strategies = fixture_strategies();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&strategies, legacy_strategy_summary));
            optimized_samples.push(measure(&strategies, strategy_summary));
        } else {
            optimized_samples.push(measure(&strategies, strategy_summary));
            legacy_samples.push(measure(&strategies, legacy_strategy_summary));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR99_EXPORT_ROW_SINGLE_BUFFER_SUMMARIES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
summaries_per_sample={SUMMARIES_PER_SAMPLE} strategies_per_summary={STRATEGIES_PER_SUMMARY} \
legacy_allocations_per_summary=34 optimized_allocations_per_summary=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer export strategy summary P95 {optimized_p95_ns}ns must be at most 70% of formatted collector P95 {legacy_p95_ns}ns"
    );
}

fn fixture_strategies() -> Vec<ExportPackagingStrategy> {
    use ExportPackagingStrategy::{LibraryEmbed, NativeDynamic, SourceTemplate};

    [SourceTemplate, LibraryEmbed, NativeDynamic]
        .into_iter()
        .cycle()
        .take(STRATEGIES_PER_SUMMARY)
        .collect()
}

fn legacy_strategy_summary(strategies: &[ExportPackagingStrategy]) -> String {
    strategies
        .iter()
        .map(|strategy| format!("{strategy:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn measure(
    strategies: &[ExportPackagingStrategy],
    summarize: fn(&[ExportPackagingStrategy]) -> String,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SUMMARIES_PER_SAMPLE {
        checksum ^= black_box(summarize(black_box(strategies))).len();
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
