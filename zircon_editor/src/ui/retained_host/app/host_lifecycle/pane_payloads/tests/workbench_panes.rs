#[test]
fn diagnostics_payload_reuses_the_publication_time_target() {
    let source = include_str!("../workbench_panes.rs");
    let function = source
        .split("fn collect_runtime_diagnostics_payload")
        .nth(1)
        .and_then(|tail| tail.split("fn collect_module_plugins_pane_payload").next())
        .expect("diagnostics payload collector");

    assert!(function.contains("self.runtime_diagnostics_refresh_target"));
    assert!(function.contains("should_collect_payload()"));
    assert!(!function.contains("RuntimeDiagnosticsRefreshTarget::None"));
    assert!(!function.contains("runtime_diagnostics_refresh_target("));
    assert!(!function.contains("tool_windows"));
}
