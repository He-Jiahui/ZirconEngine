use std::collections::BTreeMap;

use slint::Model;
use zircon_runtime::core::math::UVec2;

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, HierarchyPaneViewData, PaneContentSize, PanePayloadBuildContext,
    PanePresentation, PaneShellPresentation, SceneNodeData,
};
use crate::ui::slint_host::to_host_contract_hierarchy_pane_from_host_pane;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, ProjectOverviewSnapshot, SceneEntry,
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
        scene_entries: vec![
            SceneEntry {
                id: 7,
                name: "Root".to_string(),
                depth: 0,
                selected: true,
            },
            SceneEntry {
                id: 8,
                name: "Camera".to_string(),
                depth: 1,
                selected: false,
            },
        ],
        inspector: None,
        status_line: "Hierarchy ready".to_string(),
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

fn hierarchy_pane() -> crate::ui::layouts::windows::workbench_host_window::PaneData {
    let chrome = chrome_fixture();
    let pane_presentation = PanePresentation::new(
        PaneShellPresentation::new(
            "Hierarchy",
            "hierarchy",
            "2 nodes",
            "Hierarchy selection drives Scene and Inspector",
            None,
            false,
            blank_viewport_chrome(),
        ),
        build_pane_body_presentation(
            &PaneBodySpec::new(
                "pane.hierarchy.body",
                PanePayloadKind::HierarchyV1,
                PaneRouteNamespace::Selection,
                PaneInteractionMode::HybridNativeSlot,
            ),
            &PanePayloadBuildContext::new(&chrome),
        ),
    );

    crate::ui::layouts::windows::workbench_host_window::PaneData {
        id: "editor.hierarchy#1".into(),
        slot: "left".into(),
        kind: "Hierarchy".into(),
        title: "Hierarchy".into(),
        icon_key: "hierarchy".into(),
        subtitle: "2 nodes".into(),
        info: "Hierarchy selection drives Scene and Inspector".into(),
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
            hierarchy: HierarchyPaneViewData {
                nodes: slint::ModelRc::default(),
                hierarchy_nodes: slint::ModelRc::from(
                    &[SceneNodeData {
                        id: "legacy".into(),
                        name: "Legacy".into(),
                        depth: 0,
                        selected: false,
                    }][..],
                ),
            },
            inspector: Default::default(),
            console: Default::default(),
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
fn hierarchy_template_body_projects_hybrid_slot_and_payload_nodes_for_slint_conversion() {
    let projected = to_host_contract_hierarchy_pane_from_host_pane(
        &hierarchy_pane(),
        PaneContentSize::new(300.0, 240.0),
    );

    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert!(
        nodes
            .iter()
            .any(|node| node.control_id == "HierarchyTreeSlotAnchor"),
        "projected controls: {:?}",
        nodes
            .iter()
            .map(|node| node.control_id.to_string())
            .collect::<Vec<_>>()
    );
    assert!(nodes.iter().any(|node| node.control_id == "SelectRoot"));

    let hierarchy_nodes = (0..projected.hierarchy_nodes.row_count())
        .filter_map(|row| projected.hierarchy_nodes.row_data(row))
        .collect::<Vec<_>>();
    assert_eq!(hierarchy_nodes.len(), 2);
    assert_eq!(hierarchy_nodes[0].id, "7");
    assert_eq!(hierarchy_nodes[0].name, "Root");
    assert!(hierarchy_nodes[0].selected);
}
