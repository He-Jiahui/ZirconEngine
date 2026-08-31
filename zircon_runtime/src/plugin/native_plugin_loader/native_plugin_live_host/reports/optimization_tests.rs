use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::plugin::native::NativePluginBehaviorCallReport;

use super::{
    NativePluginRuntimeBehaviorCall, NativePluginRuntimeCommandDispatchReport,
    NativePluginRuntimeStateRestoreReport,
};

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 256;
const CALL_COUNT: usize = 4_096;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn measure_samples(mut operation: impl FnMut()) -> Vec<Duration> {
    (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            operation();
            started.elapsed()
        })
        .collect()
}

fn fixture_calls() -> Vec<NativePluginRuntimeBehaviorCall> {
    (0..CALL_COUNT)
        .map(|index| NativePluginRuntimeBehaviorCall {
            plugin_id: format!("plugin-{index:04}"),
            report: NativePluginBehaviorCallReport {
                status_code: 1,
                diagnostics: vec![format!("diagnostic-{index:04}")],
                payload: None,
            },
        })
        .collect()
}

fn fixture_dispatch_report() -> NativePluginRuntimeCommandDispatchReport {
    NativePluginRuntimeCommandDispatchReport {
        command_name: "tick".to_string(),
        calls: fixture_calls(),
        diagnostics: vec!["base diagnostic".to_string()],
    }
}

fn fixture_restore_report() -> NativePluginRuntimeStateRestoreReport {
    NativePluginRuntimeStateRestoreReport {
        calls: fixture_calls(),
        skipped_plugin_ids: Vec::new(),
        diagnostics: vec!["base diagnostic".to_string()],
    }
}

fn legacy_dispatch_diagnostics(report: &NativePluginRuntimeCommandDispatchReport) -> Vec<String> {
    let mut diagnostics = report.diagnostics.clone();
    for call in &report.calls {
        diagnostics.extend(call.report.diagnostics.iter().map(|diagnostic| {
            format!(
                "runtime plugin {} {}: {diagnostic}",
                call.plugin_id, report.command_name
            )
        }));
    }
    diagnostics.sort_unstable();
    diagnostics.dedup();
    diagnostics
}

fn legacy_restore_diagnostics(report: &NativePluginRuntimeStateRestoreReport) -> Vec<String> {
    let mut diagnostics = report.diagnostics.clone();
    for call in &report.calls {
        diagnostics.extend(call.report.diagnostics.iter().map(|diagnostic| {
            format!(
                "runtime plugin {} restore-state: {diagnostic}",
                call.plugin_id
            )
        }));
    }
    diagnostics.sort_unstable();
    diagnostics.dedup();
    diagnostics
}

#[test]
fn runtime58_preallocated_report_diagnostics_preserve_output() {
    let dispatch = fixture_dispatch_report();
    assert_eq!(
        dispatch.combined_diagnostics(),
        legacy_dispatch_diagnostics(&dispatch)
    );

    let restore = fixture_restore_report();
    assert_eq!(
        restore.combined_diagnostics(),
        legacy_restore_diagnostics(&restore)
    );
}

#[test]
fn runtime58_preallocated_report_diagnostics_source_contract() {
    let source = include_str!("../reports.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .expect("optimization test module should follow production")
        .0;
    assert_eq!(production.matches("Vec::with_capacity(").count(), 2);
    assert!(production.contains("report_diagnostic_capacity(&self.calls)"));
    assert!(!production.contains("let mut diagnostics = self.diagnostics.clone();"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_preallocated_command_diagnostics_bench() {
    let report = fixture_dispatch_report();
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_dispatch_diagnostics(&report));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(report.combined_diagnostics());
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_PREALLOCATED_COMMAND_DIAGNOSTICS_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} calls={} base_diagnostics=1 capacity=0->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        CALL_COUNT,
        CALL_COUNT + 1,
    );
    assert_eq!(report.combined_diagnostics().len(), CALL_COUNT + 1);
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime58_preallocated_restore_diagnostics_bench() {
    let report = fixture_restore_report();
    let legacy = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(legacy_restore_diagnostics(&report));
        }
    });
    let optimized = measure_samples(|| {
        for _ in 0..ITERATIONS {
            black_box(report.combined_diagnostics());
        }
    });
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "RUNTIME58_PREALLOCATED_RESTORE_DIAGNOSTICS_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} calls={} base_diagnostics=1 capacity=0->{}",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        CALL_COUNT,
        CALL_COUNT + 1,
    );
    assert_eq!(report.combined_diagnostics().len(), CALL_COUNT + 1);
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
