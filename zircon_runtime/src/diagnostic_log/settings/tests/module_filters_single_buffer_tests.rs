//! 以稳定文本和基准契约锁定模块过滤摘要；实现必须保持规则顺序、Unicode 前缀和空配置语义。
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::{diagnostic_log_module_filters, DiagnosticLogSettings};
use crate::diagnostic_log::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel, DiagnosticLogModuleFilter,
};

const PERFORMANCE_MARKER: &str = "RUNTIME870_DIAGNOSTIC_MODULE_FILTERS_SINGLE_BUFFER_BENCH_V1";
const SAMPLE_PAIRS: usize = 101;
const SUMMARIES_PER_SAMPLE: usize = 128;

#[test]
fn runtime870_diagnostic_module_filters_single_buffer_preserves_exact_text() {
    let empty =
        DiagnosticLogFilterConfig::new(DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log));
    assert_eq!(diagnostic_log_module_filters(&empty), "none");

    let config = DiagnosticLogFilterConfig {
        minimum: DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn),
        module_filters: vec![
            DiagnosticLogModuleFilter {
                scope_prefix: "zircon_runtime::asset".to_string(),
                filter: DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug),
            },
            DiagnosticLogModuleFilter {
                scope_prefix: "\u{6e32}\u{67d3}::editor".to_string(),
                filter: DiagnosticLogFilter::Off,
            },
            DiagnosticLogModuleFilter {
                scope_prefix: String::new(),
                filter: DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Error),
            },
        ],
    };
    let expected = legacy_diagnostic_log_module_filters(&config);
    assert_eq!(
        expected,
        "zircon_runtime::asset=debug,\u{6e32}\u{67d3}::editor=off,=error"
    );
    assert_eq!(diagnostic_log_module_filters(&config), expected);
    assert_eq!(
        DiagnosticLogSettings::new("runtime")
            .with_filter(config)
            .diagnostic_lines()[3],
        "diagnostic_log.module_filters=zircon_runtime::asset=debug,\u{6e32}\u{67d3}::editor=off,=error"
    );
}

#[test]
#[ignore = "release-only diagnostic module-filter performance gate"]
fn runtime870_diagnostic_module_filters_single_buffer_release_performance() {
    let config = DiagnosticLogFilterConfig {
        minimum: DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log),
        module_filters: (0..64)
            .map(|index| DiagnosticLogModuleFilter {
                scope_prefix: format!("zircon_runtime::diagnostic::subsystem_{index:04}::worker"),
                filter: match index % 3 {
                    0 => DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug),
                    1 => DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Warn),
                    _ => DiagnosticLogFilter::Off,
                },
            })
            .collect(),
    };
    assert_eq!(
        diagnostic_log_module_filters(&config),
        legacy_diagnostic_log_module_filters(&config)
    );

    for _ in 0..8 {
        black_box(render_batch(&config, legacy_diagnostic_log_module_filters));
        black_box(render_batch(&config, diagnostic_log_module_filters));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                render_batch(&config, legacy_diagnostic_log_module_filters)
            }));
            optimized_samples.push(measure(|| {
                render_batch(&config, diagnostic_log_module_filters)
            }));
        } else {
            optimized_samples.push(measure(|| {
                render_batch(&config, diagnostic_log_module_filters)
            }));
            legacy_samples.push(measure(|| {
                render_batch(&config, legacy_diagnostic_log_module_filters)
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let legacy_p99_ns = percentile_ns(&mut legacy_samples, 99);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    let optimized_p99_ns = percentile_ns(&mut optimized_samples, 99);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p99_ns={optimized_p99_ns} sample_pairs={SAMPLE_PAIRS} summaries_per_sample={SUMMARIES_PER_SAMPLE} filters_per_summary=64 legacy_child_strings_per_sample=8192 optimized_child_strings_per_sample=0 legacy_vector_slots_per_sample=8192 optimized_vector_slots_per_sample=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(110),
        "single-buffer P95 {optimized_p95_ns}ns must be at most 110% of collect/join P95 {legacy_p95_ns}ns"
    );
}

fn legacy_diagnostic_log_module_filters(config: &DiagnosticLogFilterConfig) -> String {
    if config.module_filters.is_empty() {
        "none".to_string()
    } else {
        config
            .module_filters
            .iter()
            .map(|rule| format!("{}={}", rule.scope_prefix, rule.filter))
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn render_batch(
    config: &DiagnosticLogFilterConfig,
    render: fn(&DiagnosticLogFilterConfig) -> String,
) -> usize {
    (0..SUMMARIES_PER_SAMPLE)
        .map(|_| black_box(render(black_box(config))).len())
        .sum()
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
