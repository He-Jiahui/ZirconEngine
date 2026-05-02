use std::collections::BTreeMap;

use slint::Model;
use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, ConsolePaneViewData, PaneContentSize, PanePayloadBuildContext,
    PanePresentation, PaneShellPresentation,
};
use crate::ui::slint_host::to_host_contract_console_pane_from_host_pane;
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
    }
}

fn console_pane() -> crate::ui::layouts::windows::workbench_host_window::PaneData {
    let chrome = chrome_fixture();
    let pane_presentation = PanePresentation::new(
        PaneShellPresentation::new(
            "Console",
            "console",
            "Task Output",
            chrome.status_line.clone(),
            None,
            false,
            blank_viewport_chrome(),
        ),
        build_pane_body_presentation(
            &PaneBodySpec::new(
                "pane.console.body",
                PanePayloadKind::ConsoleV1,
                PaneRouteNamespace::Dock,
                PaneInteractionMode::TemplateOnly,
            ),
            &PanePayloadBuildContext::new(&chrome),
        ),
    );

    crate::ui::layouts::windows::workbench_host_window::PaneData {
        id: "editor.console#1".into(),
        slot: "bottom_left".into(),
        kind: "Console".into(),
        title: "Console".into(),
        icon_key: "console".into(),
        subtitle: "Task Output".into(),
        info: chrome.status_line.clone().into(),
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
        native_body: crate::ui::layouts::windows::workbench_host_window::PaneNativeBodyData {
            hierarchy: Default::default(),
            inspector: Default::default(),
            console: ConsolePaneViewData {
                nodes: slint::ModelRc::default(),
                status_text: "legacy status".into(),
            },
            assets_activity: Default::default(),
            asset_browser: Default::default(),
            project_overview: Default::default(),
            module_plugins: Default::default(),
            ui_asset: Default::default(),
            animation: Default::default(),
        },
        pane_presentation: Some(pane_presentation),
    }
}

#[test]
fn console_template_body_projection_replaces_legacy_console_nodes_for_slint_conversion() {
    let projected = to_host_contract_console_pane_from_host_pane(
        &console_pane(),
        PaneContentSize::new(320.0, 180.0),
    );

    assert_eq!(projected.status_text, "Console ready");
    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    let body_section = nodes
        .iter()
        .find(|node| node.control_id == "ConsoleBodySection")
        .expect("console body section node");
    assert!(body_section.frame.width > 0.0);
    assert!(body_section.frame.height > 0.0);
    assert!(!nodes
        .iter()
        .any(|node| node.control_id == "ConsoleTextPanel"));
}
