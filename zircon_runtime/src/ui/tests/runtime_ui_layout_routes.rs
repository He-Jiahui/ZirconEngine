use std::collections::BTreeMap;

use crate::core::math::UVec2;
use crate::ui::{
    surface::{hit_test_surface_frame, UiSurface},
    RuntimeUiFixture, RuntimeUiManager,
};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiInputRoutePolicy, UiInputSequence, UiNavigationDispatchEffect, UiPointerDispatchEffect,
        UiPointerEvent, UiWindowId,
    },
    event_ui::UiNodeId,
    layout::{
        UiLayoutEngineBackend, UiLayoutEngineFallbackReason, UiLayoutEngineFamily,
        UiLayoutEngineSupport, UiPoint, UiSize,
    },
    surface::{
        UiNavigationEventKind, UiPointerButton, UiPointerEventKind, UiSurfaceDebugOptions,
        UiSurfaceDebugSnapshot,
    },
    window::{
        UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputContext,
        UiWindowInputPumpBatch, UiWindowInputPumpEvent, UiWindowMetrics, UiWindowPixelSize,
        UiWindowPlatformInputEvent,
    },
};

#[test]
fn runtime_quest_log_fixture_exports_layout_engine_route_report() {
    let mut manager = RuntimeUiManager::new(UVec2::new(960, 540));
    manager
        .load_builtin_fixture(RuntimeUiFixture::QuestLogDialog)
        .expect("quest log runtime fixture should load");

    assert_eq!(
        manager.active_fixture(),
        Some(RuntimeUiFixture::QuestLogDialog)
    );

    let surface = manager.surface();
    let dialog_id = node_id_by_control_id(surface, "QuestLogDialog");
    let actions_id = node_id_by_control_id(surface, "QuestLogActions");
    let report = surface.layout_engine_report.clone();

    assert!(report.request_count >= 3, "{report:#?}");
    assert!(report.taffy_selected_count >= 2, "{report:#?}");
    assert!(report.zircon_selected_count >= 1, "{report:#?}");
    assert!(report.fallback_count >= 1, "{report:#?}");
    assert_eq!(report.unsupported_count, 0, "{report:#?}");
    assert!(
        report.selections.iter().any(|selection| {
            selection.node_id == Some(dialog_id)
                && selection.request.family == UiLayoutEngineFamily::Flex
                && selection.selected_backend == UiLayoutEngineBackend::Taffy
                && selection.support == UiLayoutEngineSupport::Native
        }),
        "{report:#?}"
    );
    assert!(
        report.selections.iter().any(|selection| {
            selection.node_id == Some(actions_id)
                && selection.request.family == UiLayoutEngineFamily::Flex
                && selection.selected_backend == UiLayoutEngineBackend::Taffy
                && selection.support == UiLayoutEngineSupport::Native
        }),
        "{report:#?}"
    );
    assert!(
        report.selections.iter().any(|selection| {
            selection.request.family == UiLayoutEngineFamily::Overlay
                && selection.selected_backend == UiLayoutEngineBackend::Zircon
                && selection.support == UiLayoutEngineSupport::Fallback
                && selection.fallback_reason
                    == Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
        }),
        "{report:#?}"
    );

    assert_control_surface_frame_authority(surface, "QuestLogDialog");
    assert_control_surface_frame_authority(surface, "QuestLogActions");
    assert_route_report_exported(surface);
    assert_pointer_route_authority(&mut manager, "TrackQuestButton");
    assert_pointer_route_authority(&mut manager, "CloseQuestLogButton");
    assert_public_runtime_frame_uses_surface_render_extract(&manager);
}

#[test]
fn runtime_inventory_fixture_reports_virtualized_list_zircon_fallback() {
    let mut manager = RuntimeUiManager::new(UVec2::new(960, 540));
    manager
        .load_builtin_fixture(RuntimeUiFixture::InventoryList)
        .expect("inventory runtime fixture should load");

    assert_eq!(
        manager.active_fixture(),
        Some(RuntimeUiFixture::InventoryList)
    );

    let surface = manager.surface();
    let inventory_list_id = node_id_by_control_id(surface, "InventoryList");
    let report = surface.layout_engine_report.clone();

    assert!(report.request_count >= 2, "{report:#?}");
    assert!(report.zircon_selected_count >= 2, "{report:#?}");
    assert!(report.fallback_count >= 2, "{report:#?}");
    assert_eq!(report.unsupported_count, 0, "{report:#?}");
    assert!(
        report.selections.iter().any(|selection| {
            selection.request.family == UiLayoutEngineFamily::Overlay
                && selection.selected_backend == UiLayoutEngineBackend::Zircon
                && selection.support == UiLayoutEngineSupport::Fallback
                && selection.fallback_reason
                    == Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
        }),
        "{report:#?}"
    );
    assert!(
        report.selections.iter().any(|selection| {
            selection.node_id == Some(inventory_list_id)
                && selection.request.family == UiLayoutEngineFamily::VirtualizedList
                && selection.selected_backend == UiLayoutEngineBackend::Zircon
                && selection.support == UiLayoutEngineSupport::Fallback
                && selection.fallback_reason
                    == Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
        }),
        "{report:#?}"
    );

    assert_control_surface_frame_authority(surface, "InventoryList");
    assert_control_surface_frame_authority(surface, "InventoryRow00");
    assert_route_report_exported(surface);
    assert_pointer_route_authority(&mut manager, "InventoryRow00");
    assert_public_runtime_frame_uses_surface_render_extract(&manager);
}

#[test]
fn runtime_ui_manager_dispatches_window_pump_through_shared_surface() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let mut batch = UiWindowInputPumpBatch::default();
    batch.push(UiWindowInputPumpEvent::Window(
        UiWindowEvent::window_focused(window_metadata(1), true),
    ));
    batch.push(UiWindowInputPumpEvent::Window(
        UiWindowEvent::application_activation_changed(window_metadata(2), true),
    ));
    batch.push(UiWindowInputPumpEvent::Window(UiWindowEvent::new(
        window_metadata(3),
        UiWindowEventKind::Occluded { occluded: true },
    )));
    batch.push(UiWindowInputPumpEvent::Window(UiWindowEvent::window_close(
        window_metadata(4),
    )));

    let results = manager
        .dispatch_window_input_pump_batch(batch)
        .expect("runtime UI manager should route window pump batch");

    assert_eq!(results.len(), 4);
    assert_eq!(manager.surface().window_state.focused, Some(true));
    assert_eq!(
        manager.surface().window_state.application_active,
        Some(true)
    );
    assert_eq!(manager.surface().window_state.occluded, Some(true));
    assert!(manager.surface().window_state.close_requested);
    assert_eq!(
        manager.surface().surface_frame().window_state,
        manager.surface().window_state
    );
    for result in &results {
        assert_eq!(
            result.diagnostics.route_policy,
            UiInputRoutePolicy::DefaultAction
        );
        assert!(result
            .diagnostics
            .notes
            .iter()
            .any(|note| note == "window_input_pump"));
    }
    assert!(results[0]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_focus_gained"));
    assert!(results[1]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_application_active"));
    assert!(results[2]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_occluded"));
    assert!(results[3]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_close_requested"));
}

#[test]
fn runtime_ui_manager_rebuilds_after_window_pump_layout_metrics() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::QuestLogDialog)
        .expect("quest log runtime fixture should load");

    let resized_metrics = UiWindowMetrics::new(
        UiSize::new(800.0, 450.0),
        UiWindowPixelSize::new(1600, 900),
        2.0,
    );
    let result = manager
        .dispatch_window_input_pump_event(UiWindowInputPumpEvent::Window(
            UiWindowEvent::size_changed(window_metadata(5), resized_metrics),
        ))
        .expect("runtime UI manager should route window resize pump event");

    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_layout_metrics_dirty"));
    assert_eq!(
        manager
            .surface()
            .surface_frame()
            .window_state
            .metrics
            .as_ref()
            .map(|metrics| metrics.scale_factor),
        Some(2.0)
    );
    assert_eq!(manager.build_frame().viewport_size, UVec2::new(800, 450));
    assert!(
        manager.surface().last_rebuild_report.layout_recomputed,
        "layout metrics should rebuild the runtime UI surface before frame capture"
    );
    assert!(
        manager.surface().last_rebuild_report.dirty_flags.layout
            && manager.surface().last_rebuild_report.dirty_flags.hit_test
            && manager.surface().last_rebuild_report.dirty_flags.render,
        "window metrics should preserve structured layout/hit/render dirty domains"
    );
    assert!(
        !manager.surface().dirty_flags().any(),
        "runtime UI manager should consume window-pump dirty domains before the next runtime frame"
    );
}

#[test]
fn runtime_ui_manager_window_pump_batch_rebuilds_before_followup_pointer_input() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let resume_button = node_id_by_control_id(manager.surface(), "ResumeButton");
    manager.register_pointer_handler(resume_button, UiPointerEventKind::Down, move |context| {
        assert_eq!(
            context.route.hit_path.target,
            Some(resume_button),
            "pointer input in the same window-pump batch should see the resized layout"
        );
        UiPointerDispatchEffect::handled()
    });

    let resized_metrics = UiWindowMetrics::new(
        UiSize::new(800.0, 450.0),
        UiWindowPixelSize::new(1600, 900),
        2.0,
    );
    let resized_resume_point = UiPoint::new(500.0, 219.0);
    assert_ne!(
        manager.surface().hit_test(resized_resume_point).top_hit,
        Some(resume_button),
        "pre-resize hit test should not already target the resized button point"
    );

    let mut batch = UiWindowInputPumpBatch::default();
    batch.push(UiWindowInputPumpEvent::Window(UiWindowEvent::size_changed(
        window_metadata(6),
        resized_metrics,
    )));
    batch.push(UiWindowInputPumpEvent::Input(
        UiWindowPlatformInputEvent::mouse_button_down(
            UiWindowInputContext::default(),
            UiPointerButton::Primary,
            resized_resume_point,
        )
        .normalize(),
    ));

    let results = manager
        .dispatch_window_input_pump_batch(batch)
        .expect("runtime UI manager should rebuild between ordered pump events");

    assert_eq!(results.len(), 2);
    assert!(results[0]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_layout_metrics_dirty"));
    assert_eq!(results[1].diagnostics.route_target, Some(resume_button));
    assert_eq!(results[1].reply.handler, Some(resume_button));
    assert_eq!(manager.build_frame().viewport_size, UVec2::new(800, 450));
    assert!(!manager.surface().dirty_flags().any());
}

#[test]
fn runtime_ui_manager_window_pump_batch_reports_failing_index() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let root_node = manager.surface().tree.roots[0];
    manager.register_navigation_handler(root_node, UiNavigationEventKind::Activate, |_| {
        UiNavigationDispatchEffect::focus(UiNodeId::new(u64::MAX))
    });

    let resized_metrics = UiWindowMetrics::new(
        UiSize::new(800.0, 450.0),
        UiWindowPixelSize::new(1600, 900),
        2.0,
    );
    let mut batch = UiWindowInputPumpBatch::default();
    batch.push(UiWindowInputPumpEvent::Window(UiWindowEvent::size_changed(
        window_metadata(7),
        resized_metrics,
    )));
    batch.push(UiWindowInputPumpEvent::Input(
        UiWindowPlatformInputEvent::navigation(
            UiWindowInputContext::default(),
            UiNavigationEventKind::Activate,
        )
        .normalize(),
    ));

    let err = match manager.dispatch_window_input_pump_batch(batch) {
        Ok(_) => panic!("window-pump batch should report the failing input event"),
        Err(err) => err,
    };

    let message = err.to_string();
    assert!(message.contains("window input pump batch index 1"));
    assert!(message.contains("ui tree is missing node"));
    assert_eq!(manager.build_frame().viewport_size, UVec2::new(800, 450));
    assert_eq!(
        manager
            .surface()
            .surface_frame()
            .window_state
            .metrics
            .as_ref()
            .map(|metrics| metrics.scale_factor),
        Some(2.0),
        "accepted window metrics should survive the later pump input failure"
    );
    assert!(!manager.surface().dirty_flags().any());
}

fn assert_route_report_exported(surface: &crate::ui::surface::UiSurface) {
    assert_route_report_counts_and_reasons(surface);
    let frame = surface.surface_frame();
    assert_eq!(frame.layout_engine_report, surface.layout_engine_report);
    let snapshot = surface.debug_snapshot();
    assert_eq!(
        snapshot.layout_engine_report,
        surface.layout_engine_report.clone()
    );
    let snapshot_json = surface
        .debug_snapshot_json(&UiSurfaceDebugOptions::default())
        .expect("runtime fixture debug snapshot should export as JSON");
    let exported_snapshot: UiSurfaceDebugSnapshot =
        serde_json::from_str(&snapshot_json).expect("runtime fixture debug snapshot should decode");
    assert_eq!(
        exported_snapshot.layout_engine_report,
        surface.layout_engine_report.clone()
    );
}

fn window_metadata(sequence: u64) -> UiWindowEventMetadata {
    UiWindowEventMetadata {
        sequence: UiInputSequence::new(sequence),
        window_id: UiWindowId::new("runtime-primary"),
        ..UiWindowEventMetadata::default()
    }
}

fn assert_route_report_counts_and_reasons(surface: &crate::ui::surface::UiSurface) {
    let report = &surface.layout_engine_report;
    let taffy_count = report
        .selections
        .iter()
        .filter(|selection| selection.selected_backend == UiLayoutEngineBackend::Taffy)
        .count() as u64;
    let zircon_count = report
        .selections
        .iter()
        .filter(|selection| selection.selected_backend == UiLayoutEngineBackend::Zircon)
        .count() as u64;
    let fallback_count = report
        .selections
        .iter()
        .filter(|selection| selection.support == UiLayoutEngineSupport::Fallback)
        .count() as u64;
    let unsupported_count = report
        .selections
        .iter()
        .filter(|selection| selection.support == UiLayoutEngineSupport::Unsupported)
        .count() as u64;

    assert_eq!(
        report.request_count,
        report.selections.len() as u64,
        "{report:#?}"
    );
    assert_eq!(report.taffy_selected_count, taffy_count, "{report:#?}");
    assert_eq!(report.zircon_selected_count, zircon_count, "{report:#?}");
    assert_eq!(report.fallback_count, fallback_count, "{report:#?}");
    assert_eq!(report.unsupported_count, unsupported_count, "{report:#?}");
    assert_eq!(
        fallback_reason_counts(report),
        expected_fallback_reason_counts(report),
        "{report:#?}"
    );

    for selection in &report.selections {
        assert!(
            selection.node_id.is_some(),
            "route report should identify runtime fixture nodes: {selection:#?}"
        );
        match selection.support {
            UiLayoutEngineSupport::Native => {
                assert_eq!(
                    selection.selected_backend,
                    UiLayoutEngineBackend::Taffy,
                    "runtime fixture native routes should remain Taffy-owned: {selection:#?}"
                );
                assert_eq!(
                    selection.fallback_reason, None,
                    "native route should not carry a fallback reason: {selection:#?}"
                );
            }
            UiLayoutEngineSupport::Fallback => {
                assert_eq!(
                    selection.selected_backend,
                    UiLayoutEngineBackend::Zircon,
                    "fallback route should select Zircon explicitly: {selection:#?}"
                );
                assert!(
                    selection.fallback_reason.is_some(),
                    "fallback route must include a diagnostic reason: {selection:#?}"
                );
            }
            UiLayoutEngineSupport::Unsupported => {
                assert!(
                    selection.fallback_reason.is_some(),
                    "unsupported route must include a diagnostic reason: {selection:#?}"
                );
            }
        }
    }
}

fn expected_fallback_reason_counts(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
) -> BTreeMap<Option<UiLayoutEngineFallbackReason>, u64> {
    let mut counts = BTreeMap::new();
    for selection in &report.selections {
        if let Some(reason) = selection.fallback_reason {
            *counts.entry(Some(reason)).or_default() += 1;
        }
    }
    counts
}

fn fallback_reason_counts(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
) -> BTreeMap<Option<UiLayoutEngineFallbackReason>, u64> {
    report
        .fallback_reason_counts
        .iter()
        .map(|reason_count| (reason_count.reason, reason_count.count))
        .collect()
}

fn assert_control_surface_frame_authority(surface: &UiSurface, control_id: &str) {
    let node_id = node_id_by_control_id(surface, control_id);
    let surface_frame = surface.surface_frame();
    let arranged = surface_frame
        .arranged_tree
        .get(node_id)
        .unwrap_or_else(|| panic!("{control_id} should have an arranged frame"));
    let render = surface_frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)
        .unwrap_or_else(|| panic!("{control_id} should render from the arranged frame"));

    assert_eq!(
        render.frame, arranged.frame,
        "{control_id} render frame drifted"
    );
    assert_eq!(
        render.clip_frame,
        Some(arranged.clip_frame),
        "{control_id} render clip drifted"
    );
    assert_eq!(
        render.z_index, arranged.z_index,
        "{control_id} render z-index drifted"
    );
}

fn assert_public_runtime_frame_uses_surface_render_extract(manager: &RuntimeUiManager) {
    let frame = manager.build_frame();
    let surface = manager.surface();
    let surface_frame = surface.surface_frame();
    let ui = frame
        .ui
        .as_ref()
        .expect("runtime public frame should carry the surface UI extract");

    assert_eq!(
        ui, &surface_frame.render_extract,
        "public runtime frame UI extract drifted from UiSurfaceFrame"
    );
    assert_eq!(
        ui, &surface.render_extract,
        "public runtime frame UI extract drifted from UiSurface"
    );
    assert_eq!(ui.tree_id, surface_frame.tree_id);
    assert!(
        !ui.list.commands.is_empty(),
        "runtime fixture public frame should include rendered UI commands"
    );
}

fn assert_pointer_route_authority(manager: &mut RuntimeUiManager, control_id: &str) {
    let surface = manager.surface();
    let node_id = node_id_by_control_id(surface, control_id);
    let surface_frame = surface.surface_frame();
    let arranged = surface_frame
        .arranged_tree
        .get(node_id)
        .unwrap_or_else(|| panic!("{control_id} should have an arranged frame"));
    let hit = surface_frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == node_id)
        .unwrap_or_else(|| panic!("{control_id} should enter the hit grid"));

    assert_eq!(hit.frame, arranged.frame, "{control_id} hit frame drifted");
    assert_eq!(
        hit.clip_frame,
        arranged
            .frame
            .intersection(arranged.clip_frame)
            .expect("interactive runtime control should intersect its clip"),
        "{control_id} hit clip drifted"
    );
    assert_eq!(
        hit.z_index, arranged.z_index,
        "{control_id} hit z-index drifted"
    );
    assert_eq!(
        hit.paint_order, arranged.paint_order,
        "{control_id} hit paint order drifted"
    );
    assert_eq!(hit.control_id.as_deref(), Some(control_id));

    let point = UiPoint::new(
        arranged.frame.x + arranged.frame.width * 0.5,
        arranged.frame.y + arranged.frame.height * 0.5,
    );
    let frame_hit = hit_test_surface_frame(&surface_frame, point);
    assert_eq!(surface.hit_test(point), frame_hit);
    assert_eq!(frame_hit.top_hit, Some(node_id));
    assert_eq!(frame_hit.path.target, Some(node_id));
    assert_eq!(frame_hit.path.bubble_route.first().copied(), Some(node_id));

    manager.register_pointer_handler(node_id, UiPointerEventKind::Down, move |context| {
        assert_eq!(context.route.hit_path.target, Some(node_id));
        assert_eq!(
            context.route.hit_path.bubble_route.first().copied(),
            Some(node_id)
        );
        UiPointerDispatchEffect::handled()
    });

    let dispatch = manager
        .dispatch_pointer_event(
            UiPointerEvent::new(UiPointerEventKind::Down, point)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap_or_else(|error| {
            panic!("{control_id} should dispatch through pointer route: {error}")
        });

    assert_eq!(dispatch.handled_by, Some(node_id));
    assert_eq!(dispatch.route.hit_path, frame_hit.path);
    assert_eq!(dispatch.route.stacked, frame_hit.stacked);
}

fn node_id_by_control_id(surface: &crate::ui::surface::UiSurface, control_id: &str) -> UiNodeId {
    surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .unwrap_or_else(|| panic!("{control_id} should be projected"))
        .node_id
}
