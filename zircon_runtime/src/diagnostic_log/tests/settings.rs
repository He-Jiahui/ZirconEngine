use std::time::Duration;

use super::DiagnosticLogSettings;
use crate::diagnostic_log::{
    DiagnosticLogFilter, DiagnosticLogFilterConfig, DiagnosticLogLevel, DiagnosticLogLocation,
};

#[test]
fn settings_format_stable_diagnostics_for_level_filter_and_sinks() {
    let filter = DiagnosticLogFilterConfig::parse(
        "warn,zircon_runtime::asset=debug",
        DiagnosticLogFilter::Minimum(DiagnosticLogLevel::Log),
    )
    .unwrap();
    let settings = DiagnosticLogSettings::new("runtime/player")
        .with_filter(filter)
        .with_location(DiagnosticLogLocation::UnityCompatibleFirst)
        .with_console_enabled(false)
        .with_file_enabled(true);

    let diagnostics = settings.format_diagnostics();

    assert!(diagnostics.contains("diagnostic_log.channel=runtime/player"));
    assert!(diagnostics.contains("diagnostic_log.minimum=warn"));
    assert!(diagnostics.contains("diagnostic_log.filter=warn,zircon_runtime::asset=debug"));
    assert!(diagnostics.contains("diagnostic_log.module_filters=zircon_runtime::asset=debug"));
    assert!(diagnostics.contains("diagnostic_log.location=UnityCompatibleFirst"));
    assert!(diagnostics.contains("diagnostic_log.console_enabled=false"));
    assert!(diagnostics.contains("diagnostic_log.file_enabled=true"));
    assert!(diagnostics.contains("diagnostic_log.queue_capacity=4096"));
    assert!(diagnostics.contains("diagnostic_log.max_batch_records=256"));
    assert!(diagnostics.contains("diagnostic_log.max_batch_bytes=262144"));
    assert!(diagnostics.contains("diagnostic_log.flush_interval_ms=50"));
    assert!(diagnostics.contains("diagnostic_log.critical_enqueue_timeout_ms=2"));
}

#[test]
fn critical_enqueue_timeout_is_configurable_and_visible() {
    let settings = DiagnosticLogSettings::new("runtime").with_sink_settings(
        super::DiagnosticLogSinkSettings::default()
            .with_critical_enqueue_timeout(Duration::from_millis(7)),
    );

    assert_eq!(
        settings.sink.critical_enqueue_timeout,
        Duration::from_millis(7)
    );
    assert!(settings
        .format_diagnostics()
        .contains("diagnostic_log.critical_enqueue_timeout_ms=7"));
}
