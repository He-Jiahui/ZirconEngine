use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::diagnostics::{
    HostInvalidationDiagnostics, HostRefreshDiagnostics,
};
use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

use super::UiHostWindow;

#[test]
fn host_window_refresh_diagnostics_update_state_overlay_text() {
    let host = UiHostWindow::new().expect("host window should construct for state test");
    host.set_host_presentation(HostWindowPresentationData::default());

    let mut diagnostics = HostRefreshDiagnostics::default();
    diagnostics.record_present(96, false, true);
    host.set_host_refresh_diagnostics_overlay(diagnostics.with_invalidation_diagnostics(
        HostInvalidationDiagnostics {
            slow_path_rebuild_count: 2,
            render_rebuild_count: 3,
            paint_only_request_count: 4,
        },
    ));

    let presentation = host.get_host_presentation();
    let overlay = presentation.host_shell.debug_refresh_rate.as_str();
    assert!(overlay.contains("present 1"));
    assert!(overlay.contains("full 0"));
    assert!(overlay.contains("region 1"));
    assert!(overlay.contains("pixels 96"));
    assert!(overlay.contains("slow 2"));
    assert!(overlay.contains("render 3"));
    assert!(overlay.contains("paint-only 4"));
}

#[test]
fn close_requested_callback_can_mutate_host_state_without_reentrant_borrow() {
    let host = UiHostWindow::new().expect("host window should construct for state test");
    let callback_host = host.clone_strong();
    host.window().on_close_requested(move || {
        callback_host.set_host_refresh_invalidation_diagnostics(HostInvalidationDiagnostics {
            slow_path_rebuild_count: 1,
            render_rebuild_count: 2,
            paint_only_request_count: 3,
        });
        CloseRequestResponse::HideWindow
    });

    assert_eq!(
        host.close_requested_response(),
        CloseRequestResponse::HideWindow
    );
    let diagnostics = host.refresh_invalidation_diagnostics();
    assert_eq!(diagnostics.slow_path_rebuild_count, 1);
    assert_eq!(diagnostics.render_rebuild_count, 2);
    assert_eq!(diagnostics.paint_only_request_count, 3);
}

#[test]
fn frame_update_region_queues_external_redraw_with_frame_update() {
    let host = UiHostWindow::new().expect("host window should construct for redraw test");
    let frame = FrameRect {
        x: 12.0,
        y: 24.0,
        width: 128.0,
        height: 72.0,
    };

    host.request_frame_update_region(frame.clone());

    let redraw = host.take_external_redraw();
    assert!(redraw.request_redraw());
    assert!(redraw.requires_frame_update());
    assert_eq!(redraw.damage_region(), Some(&frame));
}

#[test]
fn completed_frame_update_scenario_is_one_shot() {
    let host = UiHostWindow::new().expect("host window should construct for redraw test");

    assert_eq!(host.take_completed_frame_update_scenario(), None);

    host.mark_completed_frame_update_scenario(UiPerfScenario::DrawerResize);

    assert_eq!(
        host.take_completed_frame_update_scenario(),
        Some(UiPerfScenario::DrawerResize)
    );
    assert_eq!(host.take_completed_frame_update_scenario(), None);
}
