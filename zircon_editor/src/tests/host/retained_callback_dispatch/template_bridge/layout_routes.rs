use std::collections::BTreeMap;

use super::super::support::*;
use crate::ui::template_runtime::builtin::{
    SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID, UI_HOST_WINDOW_DOCUMENT_ID,
};
use crate::ui::template_runtime::EditorUiHostRuntime;
use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher,
    surface::{hit_test_surface_frame, UiSurface},
};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchEffect,
    event_ui::UiNodeId,
    layout::{
        UiLayoutEngineBackend, UiLayoutEngineFallbackReason, UiLayoutEngineFamily,
        UiLayoutEngineSupport,
    },
    surface::{
        UiBrushPayload, UiRenderCommandKind, UiRenderResourceKind, UiSurfaceDebugOptions,
        UiSurfaceDebugSnapshot, UiVisualAssetRef,
    },
    tree::{UiInputPolicy, UiTreeNode},
};

const HOST_DRAWER_SOURCE_DOCUMENT_ID: &str = "workbench.drawer_source";
const FLOATING_WINDOW_SOURCE_DOCUMENT_ID: &str = "floating_window.source";

#[test]
fn builtin_editor_host_templates_export_layout_engine_route_reports() {
    let _guard = env_lock().lock().unwrap();
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let mut workbench = shared_surface(
        &runtime,
        UI_HOST_WINDOW_DOCUMENT_ID,
        "editor.workbench.shell",
    );
    assert_native_route(&workbench, "WorkbenchScaffold", UiLayoutEngineFamily::Flex);
    assert_native_route(&workbench, "WorkbenchBody", UiLayoutEngineFamily::Flex);
    assert_fallback_route(
        &workbench,
        "UiHostWindowRoot",
        UiLayoutEngineFamily::Overlay,
    );
    assert_fallback_route(
        &workbench,
        "LeftDrawerShellRoot",
        UiLayoutEngineFamily::Overlay,
    );
    assert_workbench_shell_frames(&workbench);
    assert_pointer_route_authority(&mut workbench, "OpenProject");
    assert_route_report_exported(&workbench);

    let drawer = shared_surface(
        &runtime,
        HOST_DRAWER_SOURCE_DOCUMENT_ID,
        "editor.workbench.drawer_source",
    );
    assert_native_route(
        &drawer,
        "WorkbenchDrawerSourceRoot",
        UiLayoutEngineFamily::Flex,
    );
    assert_native_route(
        &drawer,
        "WorkbenchDrawerCenterRowRoot",
        UiLayoutEngineFamily::Flex,
    );
    assert_native_route(&drawer, "LeftDrawerShellRoot", UiLayoutEngineFamily::Flex);
    assert_eq!(drawer.layout_engine_report.unsupported_count, 0);
    assert_drawer_source_frames(&drawer);
    assert_route_report_exported(&drawer);

    let floating = shared_surface(
        &runtime,
        FLOATING_WINDOW_SOURCE_DOCUMENT_ID,
        "editor.floating_window.source",
    );
    assert_native_route(
        &floating,
        "FloatingWindowSourceRoot",
        UiLayoutEngineFamily::Flex,
    );
    assert_native_route(
        &floating,
        "FloatingWindowCenterBandRoot",
        UiLayoutEngineFamily::Flex,
    );
    assert_eq!(floating.layout_engine_report.unsupported_count, 0);
    assert_floating_window_source_frames(&floating);
    assert_route_report_exported(&floating);

    let mut toolbar = shared_surface_with_size(
        &runtime,
        SCENE_VIEWPORT_TOOLBAR_DOCUMENT_ID,
        "editor.scene.viewport_toolbar",
        UiSize::new(1280.0, 28.0),
    );
    assert_native_route(
        &toolbar,
        "SceneViewportToolbarRoot",
        UiLayoutEngineFamily::Flex,
    );
    assert_native_route(
        &toolbar,
        "SceneViewportToolbarLeftGroup",
        UiLayoutEngineFamily::Flex,
    );
    assert_native_route(
        &toolbar,
        "SceneViewportToolbarRightGroup",
        UiLayoutEngineFamily::Flex,
    );
    assert_eq!(toolbar.layout_engine_report.unsupported_count, 0);
    assert_viewport_toolbar_frames(&toolbar);
    assert_pointer_route_authority(&mut toolbar, "FrameSelection");
    assert_pointer_route_authority(&mut toolbar, "SetProjectionMode");
    assert_route_report_exported(&toolbar);
}

fn shared_surface(runtime: &EditorUiHostRuntime, document_id: &str, tree_id: &str) -> UiSurface {
    shared_surface_with_size(runtime, document_id, tree_id, UiSize::new(1280.0, 720.0))
}

fn shared_surface_with_size(
    runtime: &EditorUiHostRuntime,
    document_id: &str,
    tree_id: &str,
    size: UiSize,
) -> UiSurface {
    let mut surface = runtime
        .build_shared_surface(document_id)
        .unwrap_or_else(|error| panic!("{document_id} should build as shared surface: {error}"));
    surface.tree.tree_id = zircon_runtime_interface::ui::event_ui::UiTreeId::new(tree_id);
    surface.compute_layout(size).unwrap();
    surface
}

fn assert_workbench_shell_frames(surface: &UiSurface) {
    assert_control_frame(
        surface,
        "UiHostWindowRoot",
        UiFrame::new(0.0, 0.0, 1280.0, 720.0),
    );
    assert_control_frame(
        surface,
        "WorkbenchScaffold",
        UiFrame::new(0.0, 0.0, 1280.0, 720.0),
    );
    assert_control_frame(
        surface,
        "WorkbenchMenuBarRoot",
        UiFrame::new(0.0, 0.0, 1280.0, 24.0),
    );
    assert_control_frame(surface, "OpenProject", UiFrame::new(0.0, 0.0, 76.0, 24.0));
    assert_control_frame(
        surface,
        "WorkbenchBody",
        UiFrame::new(0.0, 57.0, 1280.0, 639.0),
    );
    assert_control_frame(
        surface,
        "ActivityRailRoot",
        UiFrame::new(0.0, 57.0, 44.0, 639.0),
    );
    assert_control_frame(
        surface,
        "DocumentHostRoot",
        UiFrame::new(44.0, 57.0, 1236.0, 639.0),
    );
    assert_control_frame(
        surface,
        "PaneSurfaceRoot",
        UiFrame::new(44.0, 89.0, 1236.0, 607.0),
    );
    assert_control_frame(
        surface,
        "StatusBarRoot",
        UiFrame::new(0.0, 696.0, 1280.0, 24.0),
    );
    assert_control_frame(
        surface,
        "WorkbenchShellReferenceImage",
        UiFrame::new(0.0, 0.0, 1280.0, 720.0),
    );
    assert_workbench_reference_image_surface_contract(surface);
}

fn assert_drawer_source_frames(surface: &UiSurface) {
    assert_control_frame(
        surface,
        "WorkbenchDrawerSourceRoot",
        UiFrame::new(0.0, 0.0, 1280.0, 720.0),
    );
    assert_control_frame(
        surface,
        "WorkbenchDrawerTopBarRoot",
        UiFrame::new(0.0, 0.0, 1280.0, 59.0),
    );
    assert_control_frame(
        surface,
        "WorkbenchDrawerCenterRowRoot",
        UiFrame::new(0.0, 59.0, 1280.0, 637.0),
    );
    assert_control_frame(
        surface,
        "WorkbenchDrawerDocumentRoot",
        UiFrame::new(0.0, 59.0, 1280.0, 637.0),
    );
    assert_control_frame(
        surface,
        "WorkbenchDrawerStatusBarRoot",
        UiFrame::new(0.0, 696.0, 1280.0, 24.0),
    );
}

fn assert_floating_window_source_frames(surface: &UiSurface) {
    assert_control_frame(
        surface,
        "FloatingWindowSourceRoot",
        UiFrame::new(0.0, 0.0, 1280.0, 720.0),
    );
    assert_control_frame(
        surface,
        "FloatingWindowTopBarRoot",
        UiFrame::new(0.0, 0.0, 1280.0, 30.0),
    );
    assert_control_frame(
        surface,
        "FloatingWindowCenterBandRoot",
        UiFrame::new(0.0, 30.0, 1280.0, 668.0),
    );
    assert_control_frame(
        surface,
        "FloatingWindowDocumentRoot",
        UiFrame::new(34.0, 30.0, 1246.0, 668.0),
    );
    assert_control_frame(
        surface,
        "FloatingWindowStatusBarRoot",
        UiFrame::new(0.0, 698.0, 1280.0, 22.0),
    );
}

fn assert_viewport_toolbar_frames(surface: &UiSurface) {
    assert_control_frame(
        surface,
        "SceneViewportToolbarRoot",
        UiFrame::new(0.0, 0.0, 1280.0, 28.0),
    );
    assert_control_frame(
        surface,
        "SceneViewportToolbarRightGroup",
        UiFrame::new(1100.0, 0.0, 180.0, 28.0),
    );
    assert_control_frame(surface, "SetTool", UiFrame::new(0.0, 0.0, 58.0, 28.0));
    assert_control_frame(
        surface,
        "FrameSelection",
        UiFrame::new(674.0, 0.0, 68.0, 28.0),
    );
    assert_control_frame(
        surface,
        "SetProjectionMode",
        UiFrame::new(1100.0, 0.0, 88.0, 28.0),
    );
    assert_control_frame(surface, "AlignView", UiFrame::new(1192.0, 0.0, 88.0, 28.0));
}

fn assert_native_route(surface: &UiSurface, control_id: &str, family: UiLayoutEngineFamily) {
    let node_id = node_id_by_control_id(surface, control_id);
    let report = &surface.layout_engine_report;
    assert!(
        report.selections.iter().any(|selection| {
            selection.node_id == Some(node_id)
                && selection.request.family == family
                && selection.selected_backend == UiLayoutEngineBackend::Taffy
                && selection.support == UiLayoutEngineSupport::Native
        }),
        "{control_id} should route to native Taffy for {family:?}: {report:#?}"
    );
}

fn assert_fallback_route(surface: &UiSurface, control_id: &str, family: UiLayoutEngineFamily) {
    let node_id = node_id_by_control_id(surface, control_id);
    let report = &surface.layout_engine_report;
    assert!(
        report.selections.iter().any(|selection| {
            selection.node_id == Some(node_id)
                && selection.request.family == family
                && selection.selected_backend == UiLayoutEngineBackend::LegacyZircon
                && selection.support == UiLayoutEngineSupport::Fallback
                && selection.fallback_reason
                    == Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
        }),
        "{control_id} should route to Zircon fallback for {family:?}: {report:#?}"
    );
}

fn assert_route_report_exported(surface: &UiSurface) {
    let report = surface.layout_engine_report.clone();
    assert!(report.request_count > 0, "{report:#?}");
    assert_route_report_counts_and_reasons(surface);
    assert_eq!(report.unsupported_count, 0, "{report:#?}");
    assert_eq!(surface.surface_frame().layout_engine_report, report);
    assert_eq!(surface.debug_snapshot().layout_engine_report, report);
    let snapshot_json = surface
        .debug_snapshot_json(&UiSurfaceDebugOptions::default())
        .expect("editor host debug snapshot should export as JSON");
    let exported_snapshot: UiSurfaceDebugSnapshot =
        serde_json::from_str(&snapshot_json).expect("editor host debug snapshot should decode");
    assert_eq!(exported_snapshot.layout_engine_report, report);
}

fn assert_route_report_counts_and_reasons(surface: &UiSurface) {
    let report = &surface.layout_engine_report;
    let taffy_count = report
        .selections
        .iter()
        .filter(|selection| selection.selected_backend == UiLayoutEngineBackend::Taffy)
        .count() as u64;
    let legacy_count = report
        .selections
        .iter()
        .filter(|selection| selection.selected_backend == UiLayoutEngineBackend::LegacyZircon)
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
    assert_eq!(report.legacy_selected_count, legacy_count, "{report:#?}");
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
            "route report should identify editor host nodes: {selection:#?}"
        );
        match selection.support {
            UiLayoutEngineSupport::Native => {
                assert_eq!(
                    selection.selected_backend,
                    UiLayoutEngineBackend::Taffy,
                    "editor host native routes should remain Taffy-owned: {selection:#?}"
                );
                assert_eq!(
                    selection.fallback_reason, None,
                    "native route should not carry a fallback reason: {selection:#?}"
                );
            }
            UiLayoutEngineSupport::Fallback => {
                assert_eq!(
                    selection.selected_backend,
                    UiLayoutEngineBackend::LegacyZircon,
                    "fallback route should select LegacyZircon explicitly: {selection:#?}"
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

fn node_id_by_control_id(surface: &UiSurface, control_id: &str) -> UiNodeId {
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

fn node_by_control_id<'a>(surface: &'a UiSurface, control_id: &str) -> &'a UiTreeNode {
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
}

fn assert_workbench_reference_image_surface_contract(surface: &UiSurface) {
    let node = node_by_control_id(surface, "WorkbenchShellReferenceImage");
    let metadata = node
        .template_metadata
        .as_ref()
        .expect("reference image should keep template metadata");
    assert_eq!(metadata.component, "Image");
    assert_eq!(node.input_policy, UiInputPolicy::Ignore);
    assert_eq!(
        metadata
            .attributes
            .get("image")
            .and_then(toml::Value::as_str),
        Some("ui/editor/reference/workbench.png")
    );
    assert_eq!(
        metadata
            .attributes
            .get("reference_source")
            .and_then(toml::Value::as_str),
        Some("docs/ui-and-layout/workbench.png")
    );

    let surface_frame = surface.surface_frame();
    let arranged = surface_frame
        .arranged_tree
        .get(node.node_id)
        .expect("reference image should be arranged");
    assert_eq!(
        surface_frame.arranged_tree.draw_order.last().copied(),
        Some(node.node_id),
        "reference image should draw last so it is the exact visible baseline"
    );
    assert!(
        !surface_frame
            .hit_grid
            .entries
            .iter()
            .any(|entry| entry.node_id == node.node_id),
        "reference image overlay must not intercept editor input"
    );

    let render = surface_frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node.node_id)
        .expect("reference image should export a render command");
    assert_eq!(render.kind, UiRenderCommandKind::Image);
    assert_eq!(
        render.image.as_ref(),
        Some(&UiVisualAssetRef::Image(
            "ui/editor/reference/workbench.png".to_owned()
        ))
    );

    let paint = render.to_paint_element(0);
    let payload = match paint.payload {
        zircon_runtime_interface::ui::surface::UiPaintPayload::Brush { brushes } => {
            let Some(UiBrushPayload::Image(payload)) = brushes.fill else {
                panic!("reference image should paint as an image brush");
            };
            payload
        }
        _ => panic!("reference image should paint with a brush payload"),
    };
    assert_eq!(payload.resource.kind, UiRenderResourceKind::Image);
    assert_eq!(
        payload.resource.id.as_str(),
        "ui/editor/reference/workbench.png"
    );
    assert_eq!(payload.resource_state.pixel_size, Some((1280.0, 720.0)));
    assert_eq!(render.z_index, arranged.z_index);
}

fn assert_control_frame(surface: &UiSurface, control_id: &str, expected: UiFrame) {
    let node_id = node_id_by_control_id(surface, control_id);
    let surface_frame = surface.surface_frame();
    let arranged = surface_frame
        .arranged_tree
        .get(node_id)
        .unwrap_or_else(|| panic!("{control_id} should have an arranged frame"));
    assert_eq!(arranged.frame, expected, "{control_id} frame drifted");

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

fn assert_pointer_route_authority(surface: &mut UiSurface, control_id: &str) {
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
            .expect("interactive editor control should intersect its clip"),
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

    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(node_id, UiPointerEventKind::Down, move |context| {
        assert_eq!(context.route.hit_path.target, Some(node_id));
        assert_eq!(
            context.route.hit_path.bubble_route.first().copied(),
            Some(node_id)
        );
        UiPointerDispatchEffect::handled()
    });
    let dispatch = surface
        .dispatch_pointer_event(
            &dispatcher,
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
