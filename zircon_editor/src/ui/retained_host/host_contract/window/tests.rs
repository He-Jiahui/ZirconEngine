use crate::scene::viewport::{CapturedFrame, RenderViewportHandle};
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostDockPresentationPatch, HostWindowPresentationData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::diagnostics::{
    HostInvalidationDiagnostics, HostRefreshDiagnostics, HostWindowDiagnosticSeverity,
};
use crate::ui::retained_host::host_contract::PaneSurfaceHostContext;
use crate::ui::retained_host::primitives::{CloseRequestResponse, ModelRc, VecModel};
use crate::ui::retained_host::ui_perf::UiPerfScenario;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime::asset::project::ProjectPaths;

use super::UiHostWindow;

#[test]
fn presentation_generation_reuses_structure_until_a_structural_publish() {
    let host = UiHostWindow::new().expect("host window should construct for generation test");

    let initial = host.get_host_presentation_generation();
    let stable = host.get_host_presentation_generation();

    assert!(initial.shares_structure_with(&stable));
    assert!(initial.shares_theme_with(&stable));
    assert_eq!(
        initial.structure_generation(),
        stable.structure_generation()
    );

    host.set_host_presentation(initial.structure().clone());
    let published = host.get_host_presentation_generation();

    assert!(!initial.shares_structure_with(&published));
    assert!(published.structure_generation() > initial.structure_generation());
    assert_eq!(
        initial.hit_test_generation(),
        published.hit_test_generation()
    );
}

#[test]
fn dock_patch_mutates_the_unshared_structure_without_materializing_the_window() {
    let host = UiHostWindow::new().expect("host window should construct for dock patch test");
    let previous_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.previous".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let next_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.next".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.rail_nodes = previous_nodes.clone();
    host.set_host_presentation(presentation);

    let initial = host.get_host_presentation_generation();
    let structure_address = initial.structure() as *const HostWindowPresentationData;
    let expected_generation = initial.structure_generation();
    let mut next_dock = initial.structure().host_scene_data.left_dock.clone();
    next_dock.rail_nodes = next_nodes.clone();
    drop(initial);

    assert!(host.patch_host_presentation_dock(
        expected_generation,
        Default::default(),
        Default::default(),
        HostDockPresentationPatch::Left(next_dock),
        &[(previous_nodes, next_nodes.clone())],
    ));

    let patched = host.get_host_presentation_generation();
    assert_eq!(
        patched.structure() as *const HostWindowPresentationData,
        structure_address,
        "an unshared retained presentation must be patched in place"
    );
    assert!(patched
        .structure()
        .host_scene_data
        .left_dock
        .rail_nodes
        .shares_values_with(&next_nodes));
}

#[test]
fn dock_patch_preserves_an_outstanding_generation_snapshot() {
    let host = UiHostWindow::new().expect("host window should construct for dock patch test");
    let previous_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.previous".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let next_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.next".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.rail_nodes = previous_nodes.clone();
    host.set_host_presentation(presentation);

    let outstanding = host.get_host_presentation_generation();
    let old_address = outstanding.structure() as *const HostWindowPresentationData;
    let mut next_dock = outstanding.structure().host_scene_data.left_dock.clone();
    next_dock.rail_nodes = next_nodes.clone();

    assert!(host.patch_host_presentation_dock(
        outstanding.structure_generation(),
        Default::default(),
        Default::default(),
        HostDockPresentationPatch::Left(next_dock),
        &[(previous_nodes.clone(), next_nodes.clone())],
    ));

    assert!(outstanding
        .structure()
        .host_scene_data
        .left_dock
        .rail_nodes
        .shares_values_with(&previous_nodes));
    let patched = host.get_host_presentation_generation();
    assert_ne!(
        patched.structure() as *const HostWindowPresentationData,
        old_address,
        "an outstanding immutable generation requires copy-on-write"
    );
    assert!(patched
        .structure()
        .host_scene_data
        .left_dock
        .rail_nodes
        .shares_values_with(&next_nodes));
}

#[test]
fn hover_updates_only_the_interaction_generation_and_skip_equal_values() {
    let host = UiHostWindow::new().expect("host window should construct for generation test");
    let baseline = host.get_host_presentation_generation();
    let frame = FrameRect {
        x: 12.0,
        y: 24.0,
        width: 96.0,
        height: 20.0,
    };

    host.set_hovered_template_node_for_pointer_move("toolbar.play".into(), frame.clone());
    let hovered = host.get_host_presentation_generation();

    assert!(baseline.shares_structure_with(&hovered));
    assert_eq!(
        baseline.structure_generation(),
        hovered.structure_generation()
    );
    assert_eq!(
        baseline.hit_test_generation(),
        hovered.hit_test_generation()
    );
    assert!(hovered.interaction_generation() > baseline.interaction_generation());
    assert_eq!(
        hovered
            .materialize()
            .pane_interaction_state
            .hovered_template_control_id
            .as_str(),
        "toolbar.play"
    );

    host.set_hovered_template_node_for_pointer_move("toolbar.play".into(), frame);
    let repeated = host.get_host_presentation_generation();

    assert_eq!(
        repeated.interaction_generation(),
        hovered.interaction_generation()
    );
    assert!(hovered.shares_structure_with(&repeated));
}

#[test]
fn viewport_capture_advances_only_the_viewport_generation() {
    let host = UiHostWindow::new().expect("host window should construct for generation test");
    let baseline = host.get_host_presentation_generation();

    assert!(host
        .global::<PaneSurfaceHostContext>()
        .set_viewport_capture(
            RenderViewportHandle::new(7),
            CapturedFrame::new(1, 1, vec![255, 0, 0, 255], 11),
        ));
    let captured = host.get_host_presentation_generation();

    assert!(baseline.shares_structure_with(&captured));
    assert!(captured.structure().viewport_image.is_none());
    assert_eq!(
        baseline.structure_generation(),
        captured.structure_generation()
    );
    assert_eq!(
        baseline.interaction_generation(),
        captured.interaction_generation()
    );
    assert!(captured.viewport_generation() > baseline.viewport_generation());
    assert_eq!(
        captured
            .materialize()
            .viewport_image
            .expect("capture should materialize")
            .resource_key,
        "viewport:7:11"
    );
}

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

#[test]
fn native_resize_reflow_gate_is_explicitly_staged_and_committed() {
    let host = UiHostWindow::new().expect("host window should construct for resize gate test");

    assert!(!host.native_resize_reflow_pending());

    host.defer_native_resize_reflow();
    assert!(host.native_resize_reflow_pending());

    host.commit_native_resize_reflow();
    assert!(!host.native_resize_reflow_pending());
}

#[test]
fn first_presented_frame_exit_policy_defaults_off_and_can_be_enabled() {
    let host = UiHostWindow::new().expect("host window should construct for policy test");

    assert!(!host.exit_after_first_presented_frame());

    host.set_exit_after_first_presented_frame(true);

    assert!(host.exit_after_first_presented_frame());
}

#[test]
fn first_presented_frame_capture_writes_one_png_and_consumes_its_request() {
    let host = UiHostWindow::new().expect("host window should construct for capture test");
    let path = std::env::temp_dir().join(format!(
        "zircon-editor-first-presented-frame-{}-{}.png",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos()
    ));
    let _ = std::fs::remove_file(&path);

    let resolved_path = ProjectPaths::resolve_path(&path).unwrap();
    host.set_first_presented_frame_capture_path(Some(resolved_path.clone()));

    let written = host
        .capture_first_presented_frame()
        .expect("first frame capture should succeed");
    assert_eq!(written, Some(resolved_path));
    let png = std::fs::read(&path).expect("capture should create a PNG");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(
        host.capture_first_presented_frame()
            .expect("capture request should be consumed"),
        None
    );
    let diagnostics = host.take_host_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].severity(),
        HostWindowDiagnosticSeverity::Info
    );
    assert!(diagnostics[0]
        .message()
        .contains("editor_product_frame_capture_written"));

    std::fs::remove_file(path).expect("capture artifact should be removable");
}

#[test]
fn first_presented_frame_capture_reports_an_unwritable_parent_to_the_app_boundary() {
    let host = UiHostWindow::new().expect("host window should construct for capture test");
    let parent_file = std::env::temp_dir().join(format!(
        "zircon-editor-first-presented-frame-parent-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos()
    ));
    let _ = std::fs::remove_file(&parent_file);
    std::fs::write(&parent_file, b"not a directory")
        .expect("capture test should create a blocking parent file");
    host.set_first_presented_frame_capture_path(Some(
        ProjectPaths::resolve_path(parent_file.join("capture.png")).unwrap(),
    ));

    let error = host
        .capture_first_presented_frame()
        .expect_err("a file cannot become the capture directory");
    assert!(error
        .to_string()
        .contains("failed to create editor first-frame capture directory"));
    host.record_first_presented_frame_capture_error(&error);
    assert_eq!(
        host.take_first_presented_frame_capture_error(),
        Some(error.to_string())
    );

    std::fs::remove_file(parent_file).expect("blocking capture parent should be removable");
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

#[test]
fn window_scale_factor_defaults_to_one_and_filters_invalid_values() {
    let host = UiHostWindow::new().expect("host window should construct for scale test");
    let window = host.window();

    assert_eq!(window.scale_factor(), 1.0);

    window.set_scale_factor(1.5);

    assert_eq!(window.scale_factor(), 1.5);

    window.set_scale_factor(f32::NAN);

    assert_eq!(window.scale_factor(), 1.0);
}
