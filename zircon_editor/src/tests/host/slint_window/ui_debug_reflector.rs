use std::rc::Rc;

use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    PaneBodyPresentation, PaneData as WorkbenchPaneData, PanePayload, PaneShellPresentation,
    RuntimeDiagnosticsPanePayload,
};
use crate::ui::slint_host::{
    build_pane_template_surface_frame,
    refresh_runtime_diagnostics_debug_reflector_from_body_surface,
    to_host_contract_runtime_diagnostics_pane_from_host_pane, FrameRect,
    HostDocumentDockSurfaceData, HostWindowLayoutData, PaneData as HostContractPaneData,
    RuntimeDiagnosticsPaneData, TemplateNodeFrameData, TemplatePaneNodeData, UiHostWindow,
};
use slint::{Model, ModelRc, PhysicalSize, VecModel};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    layout::UiSize,
    surface::{UiDebugOverlayPrimitive, UiDebugOverlayPrimitiveKind},
};

#[test]
fn runtime_diagnostics_host_conversion_projects_debug_reflector_overlay_primitives() {
    let pane = runtime_diagnostics_pane_with_overlay(UiDebugOverlayPrimitive {
        kind: UiDebugOverlayPrimitiveKind::SelectedFrame,
        node_id: Some(UiNodeId::new(42)),
        frame: UiFrame::new(11.0, 13.0, 24.0, 18.0),
        label: Some("selected".to_string()),
        severity: None,
    });

    let projected =
        to_host_contract_runtime_diagnostics_pane_from_host_pane(&pane, pane_size(180.0, 120.0));

    assert_eq!(projected.overlay_primitives.row_count(), 1);
    let primitive = projected
        .overlay_primitives
        .row_data(0)
        .expect("overlay primitive should project");
    assert_eq!(primitive.kind, UiDebugOverlayPrimitiveKind::SelectedFrame);
    assert_eq!(primitive.node_id.as_str(), "42");
    assert_eq!(primitive.frame, host_frame(11.0, 13.0, 24.0, 18.0));
    assert_eq!(primitive.label.as_str(), "selected");
}

#[test]
fn runtime_diagnostics_host_conversion_keeps_payload_reflector_text_and_overlay() {
    let pane = runtime_diagnostics_pane_with_overlay(UiDebugOverlayPrimitive {
        kind: UiDebugOverlayPrimitiveKind::SelectedFrame,
        node_id: Some(UiNodeId::new(7)),
        frame: UiFrame::new(6.0, 7.0, 40.0, 18.0),
        label: Some("from-payload".to_string()),
        severity: None,
    });

    let projected =
        to_host_contract_runtime_diagnostics_pane_from_host_pane(&pane, pane_size(220.0, 120.0));

    assert!(model_texts(&projected.nodes)
        .iter()
        .any(|text| text == "summary"));
    assert_eq!(projected.overlay_primitives.row_count(), 1);
    let primitive = projected
        .overlay_primitives
        .row_data(0)
        .expect("payload overlay primitive should project");
    assert_eq!(primitive.kind, UiDebugOverlayPrimitiveKind::SelectedFrame);
    assert_eq!(primitive.node_id.as_str(), "7");
    assert_eq!(primitive.label.as_str(), "from-payload");
}

#[test]
fn runtime_diagnostics_body_refresh_preserves_active_payload_reflector() {
    let pane = runtime_diagnostics_pane_with_overlay(UiDebugOverlayPrimitive {
        kind: UiDebugOverlayPrimitiveKind::SelectedFrame,
        node_id: Some(UiNodeId::new(7)),
        frame: UiFrame::new(6.0, 7.0, 40.0, 18.0),
        label: Some("from-payload".to_string()),
        severity: None,
    });
    let mut host_pane = HostContractPaneData {
        kind: "RuntimeDiagnostics".into(),
        runtime_diagnostics: to_host_contract_runtime_diagnostics_pane_from_host_pane(
            &pane,
            pane_size(220.0, 120.0),
        ),
        ..HostContractPaneData::default()
    };
    host_pane.body_surface_frame =
        build_pane_template_surface_frame(&host_pane, UiSize::new(220.0, 120.0));

    let refreshed = refresh_runtime_diagnostics_debug_reflector_from_body_surface(
        &mut host_pane,
        pane_size(220.0, 120.0),
    );

    assert!(
        !refreshed,
        "active payload reflector should not be replaced"
    );
    assert!(model_texts(&host_pane.runtime_diagnostics.nodes)
        .iter()
        .any(|text| text == "summary"));
    assert_eq!(
        host_pane.runtime_diagnostics.overlay_primitives.row_count(),
        1
    );
    let primitive = host_pane
        .runtime_diagnostics
        .overlay_primitives
        .row_data(0)
        .expect("active payload overlay should remain");
    assert_eq!(primitive.node_id.as_str(), "7");
    assert_eq!(primitive.label.as_str(), "from-payload");
}

#[test]
fn rust_owned_host_window_snapshot_draws_debug_reflector_overlay_from_host_data() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(320, 200));

    let mut without_overlay = ui.get_host_presentation();
    without_overlay.host_layout = host_window_layout_for_test(320.0, 200.0);
    without_overlay.host_scene_data.layout = host_window_layout_for_test(320.0, 200.0);
    without_overlay.host_scene_data.document_dock = runtime_diagnostics_dock(Vec::new());
    ui.set_host_presentation(without_overlay.clone());
    let baseline = ui
        .window()
        .take_snapshot()
        .expect("baseline host painter should capture runtime diagnostics pane");

    let mut with_overlay = without_overlay;
    with_overlay.host_scene_data.document_dock =
        runtime_diagnostics_dock(vec![crate::ui::slint_host::UiDebugOverlayPrimitiveData {
            kind: UiDebugOverlayPrimitiveKind::DamageRegion,
            frame: host_frame(20.0, 18.0, 44.0, 28.0),
            label: "damage".into(),
            severity: "warning".into(),
            ..crate::ui::slint_host::UiDebugOverlayPrimitiveData::default()
        }]);
    ui.set_host_presentation(with_overlay);
    let overlay = ui
        .window()
        .take_snapshot()
        .expect("debug reflector overlay should render");

    assert!(
        changed_pixel_count(
            overlay.width(),
            baseline.as_bytes(),
            overlay.as_bytes(),
            92,
            108,
            58,
            38,
        ) > 16,
        "snapshot-derived overlay primitives should visibly modify the Runtime Diagnostics pane"
    );
}

#[test]
fn runtime_diagnostics_live_body_surface_populates_debug_reflector_rows_and_overlays() {
    let mut pane = runtime_diagnostics_dock(Vec::new()).pane;
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(220.0, 84.0));

    let refreshed = refresh_runtime_diagnostics_debug_reflector_from_body_surface(
        &mut pane,
        pane_size(220.0, 84.0),
    );

    assert!(refreshed);
    assert!(
        model_texts(&pane.runtime_diagnostics.nodes)
            .iter()
            .any(|text| text.contains("UI Debug Reflector:")
                && text.contains("nodes")
                && !text.contains("no active surface")),
        "live runtime diagnostics pane should reflect its current UiSurfaceFrame"
    );
    assert!(
        model_texts(&pane.runtime_diagnostics.nodes)
            .iter()
            .any(|text| text.contains("Focus:")),
        "focus/capture diagnostics should be projected into live reflector rows"
    );
    assert!(
        pane.runtime_diagnostics.overlay_primitives.row_count() > 0,
        "live body surface should expose snapshot overlay primitives"
    );
}

fn runtime_diagnostics_pane_with_overlay(primitive: UiDebugOverlayPrimitive) -> WorkbenchPaneData {
    WorkbenchPaneData {
        id: "runtime.diagnostics".into(),
        slot: "document".into(),
        kind: "RuntimeDiagnostics".into(),
        title: "Runtime Diagnostics".into(),
        icon_key: "diagnostics".into(),
        subtitle: "Runtime Services".into(),
        info: "Render, physics, and animation diagnostics".into(),
        show_empty: false,
        empty_title: Default::default(),
        empty_body: Default::default(),
        primary_action_label: Default::default(),
        primary_action_id: Default::default(),
        secondary_action_label: Default::default(),
        secondary_action_id: Default::default(),
        secondary_hint: Default::default(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        native_body: Default::default(),
        pane_presentation: Some(
            crate::ui::layouts::windows::workbench_host_window::PanePresentation::new(
                PaneShellPresentation::new(
                    "Runtime Diagnostics",
                    "diagnostics",
                    "Runtime Services",
                    "Render, physics, and animation diagnostics",
                    None,
                    false,
                    blank_viewport_chrome(),
                ),
                PaneBodyPresentation {
                    document_id: "pane.runtime.diagnostics.body".to_string(),
                    payload_kind: crate::ui::workbench::view::PanePayloadKind::RuntimeDiagnosticsV1,
                    route_namespace: crate::ui::workbench::view::PaneRouteNamespace::Dock,
                    interaction_mode: crate::ui::workbench::view::PaneInteractionMode::TemplateOnly,
                    payload: PanePayload::RuntimeDiagnosticsV1(RuntimeDiagnosticsPanePayload {
                        summary: "runtime".to_string(),
                        render_status: "render".to_string(),
                        physics_status: "physics".to_string(),
                        animation_status: "animation".to_string(),
                        detail_items: Vec::new(),
                        ui_debug_reflector_summary: "summary".to_string(),
                        ui_debug_reflector_nodes: Vec::new(),
                        ui_debug_reflector_details: Vec::new(),
                        ui_debug_reflector_export_status: "export".to_string(),
                        ui_debug_reflector_overlay_primitives: vec![primitive],
                        ui_debug_reflector_has_active_snapshot: true,
                    }),
                },
            ),
        ),
    }
}

fn runtime_diagnostics_dock(
    overlay_primitives: Vec<crate::ui::slint_host::UiDebugOverlayPrimitiveData>,
) -> HostDocumentDockSurfaceData {
    HostDocumentDockSurfaceData {
        region_frame: host_frame(72.0, 58.0, 220.0, 118.0),
        header_frame: host_frame(0.0, 0.0, 220.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 220.0, 84.0),
        pane: HostContractPaneData {
            kind: "RuntimeDiagnostics".into(),
            title: "Runtime Diagnostics".into(),
            runtime_diagnostics: RuntimeDiagnosticsPaneData {
                nodes: model_rc(vec![template_node(
                    "RuntimeDiagnosticsSummary",
                    "Label",
                    "Runtime diagnostics",
                    8.0,
                    8.0,
                    130.0,
                    18.0,
                )]),
                overlay_primitives: model_rc(overlay_primitives),
                preserve_payload_debug_reflector: false,
            },
            ..HostContractPaneData::default()
        },
        ..HostDocumentDockSurfaceData::default()
    }
}

fn pane_size(
    width: f32,
    height: f32,
) -> crate::ui::layouts::windows::workbench_host_window::PaneContentSize {
    crate::ui::layouts::windows::workbench_host_window::PaneContentSize::new(width, height)
}

fn host_window_layout_for_test(width: f32, height: f32) -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: host_frame(0.0, 58.0, width, height - 82.0),
        status_bar_frame: host_frame(0.0, height - 24.0, width, 24.0),
        left_region_frame: host_frame(0.0, 58.0, 72.0, height - 82.0),
        document_region_frame: host_frame(72.0, 58.0, width - 72.0, height - 82.0),
        viewport_content_frame: host_frame(88.0, 90.0, width - 104.0, height - 124.0),
        ..HostWindowLayoutData::default()
    }
}

fn template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("{control_id}.node").into(),
        control_id: control_id.into(),
        role: role.into(),
        text: text.into(),
        surface_variant: "panel".into(),
        border_width: 1.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

fn model_texts(model: &ModelRc<TemplatePaneNodeData>) -> Vec<String> {
    (0..model.row_count())
        .filter_map(|row| model.row_data(row))
        .map(|node| node.text.to_string())
        .collect()
}

fn host_frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn changed_pixel_count(
    width: u32,
    left: &[u8],
    right: &[u8],
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
) -> usize {
    let x1 = x.saturating_add(region_width).min(width);
    let y1 = y
        .saturating_add(region_height)
        .min((left.len() / 4 / width as usize) as u32)
        .min((right.len() / 4 / width as usize) as u32);
    (y..y1)
        .flat_map(|row| (x..x1).map(move |column| (column, row)))
        .filter(|(column, row)| {
            pixel(width, left, *column, *row) != pixel(width, right, *column, *row)
        })
        .count()
}

fn pixel(width: u32, bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}
