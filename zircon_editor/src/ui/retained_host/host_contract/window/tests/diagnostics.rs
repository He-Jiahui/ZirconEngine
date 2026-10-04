use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::diagnostics::{
    HostInvalidationDiagnostics, HostRefreshDiagnostics, HostWindowDiagnosticSeverity,
};

#[test]
fn host_window_refresh_diagnostics_update_state_overlay_text() {
    let host = UiHostWindow::new().expect("host window should construct for state test");
    host.set_host_presentation(HostWindowPresentationData::default());
    let baseline = host.get_host_presentation_generation();

    let mut diagnostics = HostRefreshDiagnostics::default();
    diagnostics.record_present(96, false, true);
    host.set_host_refresh_diagnostics_overlay(diagnostics.clone().with_invalidation_diagnostics(
        HostInvalidationDiagnostics {
            slow_path_rebuild_count: 2,
            render_rebuild_count: 3,
            paint_only_request_count: 4,
        },
    ));

    let updated = host.get_host_presentation_generation();
    assert!(baseline.shares_structure_with(&updated));
    assert_eq!(
        baseline.structure_generation(),
        updated.structure_generation()
    );
    assert!(updated.diagnostics_generation() > baseline.diagnostics_generation());

    let presentation = host.get_host_presentation();
    let overlay = presentation.host_shell.debug_refresh_rate.as_str();
    assert!(overlay.contains("present 1"));
    assert!(overlay.contains("full 0"));
    assert!(overlay.contains("region 1"));
    assert!(overlay.contains("pixels 96"));
    assert!(overlay.contains("slow 2"));
    assert!(overlay.contains("render 3"));
    assert!(overlay.contains("paint-only 4"));

    let generation = updated.diagnostics_generation();
    host.set_host_refresh_diagnostics_overlay(diagnostics.with_invalidation_diagnostics(
        HostInvalidationDiagnostics {
            slow_path_rebuild_count: 2,
            render_rebuild_count: 3,
            paint_only_request_count: 4,
        },
    ));
    assert_eq!(
        host.get_host_presentation_generation()
            .diagnostics_generation(),
        generation
    );
}

#[test]
fn host_window_diagnostics_preserve_fifo_order_and_severity_until_composition() {
    let host = UiHostWindow::new().expect("host window should construct for diagnostic test");

    host.record_host_diagnostic(HostWindowDiagnosticSeverity::Info, "first frame ready");
    host.record_host_diagnostic(HostWindowDiagnosticSeverity::Warning, "gpu fallback active");

    let diagnostics = host.take_host_diagnostics();
    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].message(), "first frame ready");
    assert_eq!(
        diagnostics[0].severity(),
        HostWindowDiagnosticSeverity::Info
    );
    assert_eq!(diagnostics[1].message(), "gpu fallback active");
    assert_eq!(
        diagnostics[1].severity(),
        HostWindowDiagnosticSeverity::Warning
    );
    assert!(host.take_host_diagnostics().is_empty());
}

#[test]
fn host_window_retains_the_first_fatal_event_loop_failure() {
    let host = UiHostWindow::new().expect("host window should construct for failure test");
    let callback_host = host.clone_strong();
    callback_host.report_fatal_failure(
        "editor_host_window",
        "native_window size=1280x720",
        "native window creation failed: desktop unavailable",
        "verify the desktop session can create windows and retry zircon_editor",
    );
    callback_host.report_fatal_failure(
        "editor_host_window",
        "presenter_backend=softbuffer",
        "presenter creation failed: device lost",
        "verify the graphics adapter and restart zircon_editor",
    );
    let diagnostics = host.take_host_diagnostics();
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics
        .iter()
        .all(|diagnostic| { diagnostic.severity() == HostWindowDiagnosticSeverity::Error }));
    assert!(diagnostics[0]
        .message()
        .contains("native window creation failed"));

    assert_eq!(
        host.take_fatal_failure().unwrap().to_string(),
        "editor startup diagnostic: component=editor_host_window requested=native_window size=1280x720 cause=native window creation failed: desktop unavailable recovery=verify the desktop session can create windows and retry zircon_editor"
    );
    assert!(host.take_fatal_failure().is_none());
}
