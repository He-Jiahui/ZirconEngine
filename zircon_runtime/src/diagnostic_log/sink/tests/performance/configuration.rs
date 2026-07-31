use std::path::PathBuf;
use std::time::Duration;

use super::super::super::worker::SinkRuntime;
use super::super::super::{CompiledDiagnosticLogFilter, DiagnosticLogState};
use super::output::InstrumentedSlowOutput;
use crate::diagnostic_log::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel, DiagnosticLogModuleFilter,
    DiagnosticLogSinkSettings,
};

pub(super) const QUEUE_CAPACITY: usize = 4_096;
const MAX_BATCH_RECORDS: usize = 256;

pub(super) fn test_state(
    scoped_rule_count: usize,
    output: InstrumentedSlowOutput,
) -> DiagnosticLogState {
    let filter = filter_config(scoped_rule_count);
    let sink_settings = DiagnosticLogSinkSettings::default()
        .with_queue_capacity(QUEUE_CAPACITY)
        .with_max_batch_records(MAX_BATCH_RECORDS)
        .with_flush_interval(Duration::from_millis(25));
    let sink = SinkRuntime::start(Some(Box::new(output)), false, sink_settings.clone())
        .expect("performance sink worker");
    DiagnosticLogState {
        channel: "perf-mvp-434".to_string(),
        compiled_filter: CompiledDiagnosticLogFilter::new(&filter),
        filter,
        console_enabled: false,
        file_enabled: true,
        file_path: None::<PathBuf>,
        sink_settings,
        sink: Some(sink),
    }
}

pub(super) fn active_scope(scoped_rule_count: usize) -> String {
    if scoped_rule_count == 0 {
        "runtime::diagnostic_storm".to_string()
    } else {
        format!("scope{:04}::diagnostic_storm", scoped_rule_count - 1)
    }
}

fn filter_config(scoped_rule_count: usize) -> DiagnosticLogFilterConfig {
    DiagnosticLogFilterConfig {
        minimum: DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug),
        module_filters: (0..scoped_rule_count)
            .map(|index| DiagnosticLogModuleFilter {
                scope_prefix: format!("scope{index:04}"),
                filter: DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Debug),
            })
            .collect(),
    }
}
