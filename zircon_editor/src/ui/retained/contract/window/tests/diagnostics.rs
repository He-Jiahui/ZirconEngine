use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::diagnostics::{
    HostInvalidationDiagnostics, HostRefreshDiagnostics, HostWindowDiagnosticSeverity,
};

#[test]
fn host_window_refresh_invalidation_counts_preserve_presentation_generations() {
    let host = UiHostWindow::new().expect("host window should construct for state test");
    host.set_host_presentation(HostWindowPresentationData::default());
    let baseline = host.get_host_presentation_generation();

    let invalidation = HostInvalidationDiagnostics {
        slow_path_rebuild_count: 2,
        render_rebuild_count: 3,
        paint_only_request_count: 4,
    };
    host.set_host_refresh_invalidation_diagnostics(invalidation);

    let updated = host.get_host_presentation_generation();
    assert!(baseline.shares_structure_with(&updated));
    assert_eq!(baseline.cursor(), updated.cursor());
    let observed = host.refresh_invalidation_diagnostics();
    assert_eq!(observed.slow_path_rebuild_count, 2);
    assert_eq!(observed.render_rebuild_count, 3);
    assert_eq!(observed.paint_only_request_count, 4);
    host.set_host_refresh_invalidation_diagnostics(invalidation);
    assert_eq!(
        host.get_host_presentation_generation().cursor(),
        baseline.cursor()
    );
}

#[test]
fn authored_editor_paint_omits_retired_refresh_marker_with_or_without_nodes() {
    use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
    use crate::ui::retained_host::host_contract::paint_workbench::paint_product_presentation_for_evidence;
    use crate::ui::retained_host::primitives::{ModelRc, VecModel};
    use std::rc::Rc;

    let host = UiHostWindow::new().expect("host window should construct for state test");
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: ModelRc::from(Rc::new(VecModel::from(vec![
            TemplatePaneNodeData {
                control_id: "WorkbenchWindowRoot".into(),
                ..TemplatePaneNodeData::default()
            },
        ]))),
        ..HostWindowPresentationData::default()
    };
    let mut diagnostics = HostRefreshDiagnostics::default();
    for presentation in [presentation, HostWindowPresentationData::default()] {
        let mut altered_overlay = presentation.clone();
        altered_overlay.host_shell.debug_refresh_rate = "FPS 10 | present 999".into();
        assert_eq!(
            paint_product_presentation_for_evidence(320, 200, &presentation),
            paint_product_presentation_for_evidence(320, 200, &altered_overlay),
            "empty authored nodes must not restore the retired refresh HUD",
        );
        host.set_host_presentation(presentation);
        let baseline = host.get_host_presentation_generation();
        diagnostics.record_present(96, false, true);
        host.set_host_refresh_invalidation_diagnostics(HostInvalidationDiagnostics {
            slow_path_rebuild_count: 2,
            render_rebuild_count: 3,
            paint_only_request_count: 4,
        });
        let current = host.get_host_presentation_generation();
        assert_eq!(current.cursor(), baseline.cursor());
        assert!(current.shares_structure_with(&baseline));
        assert_eq!(
            host.get_host_presentation().host_shell.debug_refresh_rate,
            baseline.structure().host_shell.debug_refresh_rate,
        );
    }

    let recorded = diagnostics.overlay_text();
    assert!(recorded.contains("present 2"));
    assert!(recorded.contains("region 2"));
    assert!(recorded.contains("pixels 192"));
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
