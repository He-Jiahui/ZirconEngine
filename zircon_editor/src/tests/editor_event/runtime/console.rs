use super::*;

#[test]
fn console_clear_builtin_binding_clears_history_without_replacing_latest_status() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_console_clear");
    runtime.runtime.set_status_line("Compiled materials");
    runtime.runtime.set_status_line("Scene ready");

    let effects =
        dispatch_builtin_template_binding(&runtime.runtime, "ConsolePaneBody/ClearConsole")
            .expect("Console clear builtin binding should exist")
            .expect("Console clear builtin binding should dispatch");

    let snapshot = runtime.runtime.editor_snapshot();
    assert_eq!(snapshot.status_line, "Scene ready");
    assert!(snapshot.console_output.is_empty());
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PRESENTATION_DATA));
    assert!(!effects.dirty_domains().requires_layout());
}

#[test]
fn console_filter_builtin_bindings_change_visible_history_and_restore_all_messages() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_console_filter");
    runtime.runtime.set_status_line("Compiled materials");
    runtime.runtime.set_status_line("Scene ready");
    let total_counts = runtime.runtime.editor_snapshot().console_output.counts();

    let error_effects =
        dispatch_builtin_template_binding(&runtime.runtime, "ConsolePaneBody/FilterError")
            .expect("Console error-filter binding should exist")
            .expect("Console error-filter binding should dispatch");
    let filtered = runtime.runtime.editor_snapshot().console_output;
    assert!(filtered.is_empty());
    assert_eq!(filtered.counts(), total_counts);
    assert_eq!(
        filtered.filter(),
        crate::core::editor_event::ConsoleMessageFilter::Error
    );
    assert!(error_effects
        .dirty_domains()
        .contains(HostInvalidationMask::PRESENTATION_DATA));

    dispatch_builtin_template_binding(&runtime.runtime, "ConsolePaneBody/FilterAll")
        .expect("Console all-filter binding should exist")
        .expect("Console all-filter binding should dispatch");
    let restored = runtime.runtime.editor_snapshot().console_output;
    assert!(restored.ends_with("Compiled materials\nScene ready"));
    assert_eq!(
        restored.filter(),
        crate::core::editor_event::ConsoleMessageFilter::All
    );
}
