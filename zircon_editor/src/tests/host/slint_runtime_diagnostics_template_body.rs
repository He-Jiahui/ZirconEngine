use std::collections::BTreeMap;

use slint::Model;
use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, PaneContentSize, PanePayloadBuildContext, PanePresentation,
    PaneShellPresentation,
};
use crate::ui::slint_host::to_host_contract_runtime_diagnostics_pane_from_host_pane;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, ProjectOverviewSnapshot, WorkbenchSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace,
};

fn chrome_fixture() -> EditorChromeSnapshot {
    EditorChromeSnapshot {
        workbench: WorkbenchSnapshot {
            active_main_page: MainPageId::workbench(),
            main_pages: Vec::new(),
            drawers: BTreeMap::new(),
            floating_windows: Vec::new(),
        },
        scene_entries: Vec::new(),
        inspector: None,
        status_line: "Console ready".to_string(),
        hovered_axis: None,
        viewport_size: UVec2::new(1280, 720),
        scene_viewport_settings: SceneViewportSettings::default(),
        mesh_import_path: String::new(),
        project_overview: ProjectOverviewSnapshot::default(),
        asset_activity: AssetWorkspaceSnapshot::default(),
        asset_browser: AssetWorkspaceSnapshot::default(),
        project_path: "sandbox-project".to_string(),
        session_mode: EditorSessionMode::Project,
        welcome: WelcomePaneSnapshot::default(),
        project_open: true,
        can_undo: true,
        can_redo: false,
        menu_overflow_mode: Default::default(),
    }
}

fn runtime_diagnostics_pane() -> crate::ui::layouts::windows::workbench_host_window::PaneData {
    let chrome = chrome_fixture();
    let pane_presentation = PanePresentation::new(
        PaneShellPresentation::new(
            "Runtime Diagnostics",
            "diagnostics",
            "Runtime Services",
            "Render, physics, and animation diagnostics",
            None,
            false,
            blank_viewport_chrome(),
        ),
        build_pane_body_presentation(
            &PaneBodySpec::new(
                "pane.runtime.diagnostics.body",
                PanePayloadKind::RuntimeDiagnosticsV1,
                PaneRouteNamespace::Dock,
                PaneInteractionMode::TemplateOnly,
            ),
            &PanePayloadBuildContext::new(&chrome),
        ),
    );

    crate::ui::layouts::windows::workbench_host_window::PaneData {
        id: "editor.runtime_diagnostics#1".into(),
        slot: "bottom_left".into(),
        kind: "RuntimeDiagnostics".into(),
        title: "Runtime Diagnostics".into(),
        icon_key: "diagnostics".into(),
        subtitle: "Runtime Services".into(),
        info: "Render, physics, and animation diagnostics".into(),
        show_empty: false,
        empty_title: "".into(),
        empty_body: "".into(),
        primary_action_label: "".into(),
        primary_action_id: "".into(),
        secondary_action_label: "".into(),
        secondary_action_id: "".into(),
        secondary_hint: "".into(),
        show_toolbar: false,
        viewport: blank_viewport_chrome(),
        native_body: Default::default(),
        pane_presentation: Some(pane_presentation),
    }
}

#[test]
fn runtime_diagnostics_template_body_projects_ui_debug_reflector_nodes() {
    let projected = to_host_contract_runtime_diagnostics_pane_from_host_pane(
        &runtime_diagnostics_pane(),
        PaneContentSize::new(420.0, 260.0),
    );
    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();

    let summary = nodes
        .iter()
        .find(|node| node.control_id == "UiDebugReflectorSummary")
        .expect("reflector summary node should project");
    assert_eq!(
        summary.text.as_str(),
        "UI Debug Reflector: no active surface snapshot"
    );

    assert!(nodes.iter().any(|node| {
        node.control_id == "UiDebugReflectorExportStatus"
            && node.text.as_str().contains("Export unavailable")
    }));
    assert!(nodes.iter().any(|node| {
        node.control_id == "UiDebugReflectorDetail"
            && node.text.as_str().contains("UiSurfaceDebugSnapshot")
    }));
    assert!(nodes.iter().any(|node| {
        node.control_id == "UiDebugReflectorSummaryText"
            && node.text.as_str().contains("no active surface")
    }));
}
