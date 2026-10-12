use super::{HostWindowDiagnostic, HostWindowDiagnosticQueue, HostWindowDiagnosticSeverity};

#[test]
fn oversized_window_diagnostic_uses_a_bounded_fallback_without_losing_severity() {
    let mut queue = HostWindowDiagnosticQueue::default();

    queue.push(HostWindowDiagnostic::new(
        HostWindowDiagnosticSeverity::Error,
        "x".repeat(9 * 1024),
    ));

    let diagnostics = queue.drain();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].severity(),
        HostWindowDiagnosticSeverity::Error
    );
    assert_eq!(
        diagnostics[0].message(),
        "editor_host_window diagnostic exceeds the bounded queue limit."
    );
}

#[test]
fn queue_evicts_oldest_entry_when_the_byte_budget_would_be_exceeded() {
    let mut queue = HostWindowDiagnosticQueue::default();
    let oldest = "a".repeat(6 * 1024);
    let latest = "b".repeat(6 * 1024);

    queue.push(HostWindowDiagnostic::new(
        HostWindowDiagnosticSeverity::Info,
        oldest,
    ));
    queue.push(HostWindowDiagnostic::new(
        HostWindowDiagnosticSeverity::Warning,
        latest.clone(),
    ));

    let diagnostics = queue.drain();
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message(), latest.as_str());
    assert_eq!(
        diagnostics[1].message(),
        "editor_host_window diagnostics_dropped=1"
    );
}

#[test]
fn queue_reports_eviction_without_retaining_unbounded_window_diagnostics() {
    let mut queue = HostWindowDiagnosticQueue::default();
    for index in 0..65 {
        queue.push(HostWindowDiagnostic::new(
            HostWindowDiagnosticSeverity::Info,
            format!("native window diagnostic {index}"),
        ));
    }

    let diagnostics = queue.drain();
    assert_eq!(diagnostics.len(), 65);
    assert_eq!(diagnostics[0].message(), "native window diagnostic 1");
    assert_eq!(
        diagnostics
            .last()
            .expect("drop report should be present")
            .message(),
        "editor_host_window diagnostics_dropped=1"
    );
    assert_eq!(
        diagnostics
            .last()
            .expect("drop report should be present")
            .severity(),
        HostWindowDiagnosticSeverity::Warning
    );
}
