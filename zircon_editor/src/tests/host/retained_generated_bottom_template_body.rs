use std::collections::BTreeMap;

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, GeneratedBottomPaneViewData, PaneNativeBodyData, PanePayload,
    PanePayloadBuildContext, PanePresentation, PaneShellPresentation,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::to_host_contract_generated_bottom_pane_from_host_pane;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::{
    AssetWorkspaceSnapshot, EditorChromeSnapshot, ProjectOverviewSnapshot, WorkbenchSnapshot,
};
use crate::ui::workbench::startup::{EditorSessionMode, WelcomePaneSnapshot};
use crate::ui::workbench::view::{
    PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace,
};
use zircon_runtime_interface::math::UVec2;

fn chrome_fixture() -> EditorChromeSnapshot {
    EditorChromeSnapshot {
        focused_document_kind: None,
        workbench: WorkbenchSnapshot {
            active_main_page: MainPageId::workbench(),
            main_pages: Vec::new(),
            drawers: BTreeMap::new(),
            floating_windows: Vec::new(),
        },
        scene_entries: Vec::new(),
        inspector: None,
        status_line: String::new(),
        status_task_progress: None,
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

fn generated_bottom_pane() -> crate::ui::layouts::windows::workbench_host_window::PaneData {
    let chrome = chrome_fixture();
    let body = build_pane_body_presentation(
        &PaneBodySpec::new(
            "res://ui/editor/host/generated_bottom_body.zui",
            PanePayloadKind::GeneratedBottomV1,
            PaneRouteNamespace::Dock,
            PaneInteractionMode::TemplateOnly,
        ),
        &PanePayloadBuildContext::new(&chrome),
    );
    let pane_presentation = PanePresentation::new(
        PaneShellPresentation::new(
            "Generated Output",
            "generated-output",
            "Generated Output",
            "Componentized generated editor feedback panels",
            None,
            false,
            blank_viewport_chrome(),
        ),
        body,
    );
    let status = match &pane_presentation.body.payload {
        PanePayload::GeneratedBottomV1(payload) => payload.status.clone(),
        unexpected => panic!("expected generated bottom payload, found {unexpected:?}"),
    };

    crate::ui::layouts::windows::workbench_host_window::PaneData {
        id: "editor.generated_bottom#1".into(),
        slot: "bottom".into(),
        kind: "GeneratedBottom".into(),
        title: "Generated Output".into(),
        icon_key: "generated-output".into(),
        subtitle: "Generated Output".into(),
        info: "Componentized generated editor feedback panels".into(),
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
        native_body: PaneNativeBodyData {
            generated_bottom: GeneratedBottomPaneViewData {
                nodes: ModelRc::default(),
                status: status.into(),
            },
            ..PaneNativeBodyData::default()
        },
        pane_presentation: Some(pane_presentation),
    }
}

#[test]
fn generated_bottom_template_body_projects_panel_nodes_for_retained_conversion() {
    let projected = to_host_contract_generated_bottom_pane_from_host_pane(
        &generated_bottom_pane(),
        crate::ui::layouts::windows::workbench_host_window::PaneContentSize::new(520.0, 180.0),
    );

    assert_eq!(
        projected.status.as_str(),
        "Generated editor feedback panels"
    );
    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert!(
        nodes
            .iter()
            .any(|node| node.control_id.as_str() == "WorkbenchGeneratedBottomPanel"),
        "generated bottom body should project the retained generated panel root"
    );
    assert!(
        nodes
            .iter()
            .any(|node| node.action_id.as_str() == "workbench.generated_bottom.open_panel.invoke"),
        "generated bottom body should preserve existing generated bottom action bindings"
    );
}
