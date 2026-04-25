use std::collections::BTreeMap;

use slint::Model;
use zircon_runtime::core::math::UVec2;

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, InspectorPaneViewData, PaneContentSize, PanePayloadBuildContext,
    PanePresentation, PaneShellPresentation,
};
use crate::ui::slint_host::to_slint_inspector_pane_from_host_pane;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, InspectorSnapshot, ProjectOverviewSnapshot,
    WorkbenchSnapshot,
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
        inspector: Some(InspectorSnapshot {
            id: 7,
            name: "Root".to_string(),
            parent: "Scene".to_string(),
            translation: ["1.0".to_string(), "2.0".to_string(), "3.0".to_string()],
        }),
        status_line: "Inspector ready".to_string(),
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
    }
}

fn inspector_pane() -> crate::ui::layouts::windows::workbench_host_window::PaneData {
    let chrome = chrome_fixture();
    let pane_presentation = PanePresentation::new(
        PaneShellPresentation::new(
            "Inspector",
            "inspector",
            "Selection Inspector",
            "Node 7",
            None,
            false,
            blank_viewport_chrome(),
        ),
        build_pane_body_presentation(
            &PaneBodySpec::new(
                "pane.inspector.body",
                PanePayloadKind::InspectorV1,
                PaneRouteNamespace::Draft,
                PaneInteractionMode::TemplateOnly,
            ),
            &PanePayloadBuildContext::new(&chrome),
        ),
    );

    crate::ui::layouts::windows::workbench_host_window::PaneData {
        id: "editor.inspector#1".into(),
        slot: "right".into(),
        kind: "Inspector".into(),
        title: "Inspector".into(),
        icon_key: "inspector".into(),
        subtitle: "Selection Inspector".into(),
        info: "Node 7".into(),
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
        body_compat: crate::ui::layouts::windows::workbench_host_window::PaneBodyCompatData {
            hierarchy: Default::default(),
            inspector: InspectorPaneViewData {
                nodes: slint::ModelRc::default(),
                info: "legacy node".into(),
                inspector_name: "Legacy".into(),
                inspector_parent: "LegacyParent".into(),
                inspector_x: "9.0".into(),
                inspector_y: "8.0".into(),
                inspector_z: "7.0".into(),
                delete_enabled: false,
            },
            console: Default::default(),
            assets_activity: Default::default(),
            asset_browser: Default::default(),
            project_overview: Default::default(),
            ui_asset: Default::default(),
            animation: Default::default(),
        },
        pane_presentation: Some(pane_presentation),
    }
}

#[test]
fn inspector_template_body_projection_replaces_legacy_inspector_view_data_for_slint_conversion() {
    let projected = to_slint_inspector_pane_from_host_pane(
        &inspector_pane(),
        PaneContentSize::new(360.0, 220.0),
    );

    assert_eq!(projected.info, "Node 7");
    assert_eq!(projected.inspector_name, "Root");
    assert_eq!(projected.inspector_parent, "Scene");
    assert_eq!(projected.inspector_x, "1.0");
    assert_eq!(projected.inspector_y, "2.0");
    assert_eq!(projected.inspector_z, "3.0");
    assert!(projected.delete_enabled);

    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    let body_section = nodes
        .iter()
        .find(|node| node.control_id == "InspectorBodySection")
        .expect("inspector body section node");
    assert!(body_section.frame.width > 0.0);
    assert!(body_section.frame.height > 0.0);
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "ApplyDraft" && node.text == "Apply Draft"));
    assert!(!nodes
        .iter()
        .any(|node| node.control_id == "InspectorContentPanel"));
}
