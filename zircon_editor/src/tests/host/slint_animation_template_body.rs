use std::collections::BTreeMap;

use slint::Model;
use zircon_runtime_interface::math::UVec2;

use crate::scene::viewport::SceneViewportSettings;
use crate::ui::animation_editor::AnimationEditorPanePresentation;
use crate::ui::layouts::views::blank_viewport_chrome;
use crate::ui::layouts::windows::workbench_host_window::{
    build_pane_body_presentation, AnimationEditorPaneViewData, PaneContentSize,
    PanePayloadBuildContext, PanePresentation, PaneShellPresentation,
};
use crate::ui::slint_host::to_host_contract_animation_editor_pane_from_host_pane;
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
        status_line: "Animation ready".to_string(),
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

fn animation_fixture(mode: &str) -> AnimationEditorPanePresentation {
    AnimationEditorPanePresentation {
        mode: mode.to_string(),
        asset_path: "res://animations/hero.anim".to_string(),
        status: "Ready".to_string(),
        selection_summary: "Track Root/Hero selected".to_string(),
        current_frame: 12,
        timeline_start_frame: 5,
        timeline_end_frame: 42,
        playback_label: "Paused".to_string(),
        track_items: vec!["Root/Hero:Transform.position".to_string()],
        parameter_items: vec!["speed".to_string()],
        node_items: vec!["Blend".to_string()],
        state_items: vec!["Idle".to_string(), "Run".to_string()],
        transition_items: vec!["Idle -> Run".to_string()],
    }
}

fn animation_pane(
    document_id: &str,
    payload_kind: PanePayloadKind,
    route_namespace: PaneRouteNamespace,
    animation: &AnimationEditorPanePresentation,
) -> crate::ui::layouts::windows::workbench_host_window::PaneData {
    let chrome = chrome_fixture();
    let pane_presentation = PanePresentation::new(
        PaneShellPresentation::new(
            "Animation",
            "animation",
            animation.asset_path.clone(),
            animation.status.clone(),
            None,
            false,
            blank_viewport_chrome(),
        ),
        build_pane_body_presentation(
            &PaneBodySpec::new(
                document_id,
                payload_kind,
                route_namespace,
                PaneInteractionMode::HybridNativeSlot,
            ),
            &PanePayloadBuildContext::new(&chrome).with_animation_pane(animation),
        ),
    );

    crate::ui::layouts::windows::workbench_host_window::PaneData {
        id: "editor.animation#1".into(),
        slot: "".into(),
        kind: if payload_kind == PanePayloadKind::AnimationSequenceV1 {
            "AnimationSequenceEditor".into()
        } else {
            "AnimationGraphEditor".into()
        },
        title: "Animation".into(),
        icon_key: "animation".into(),
        subtitle: animation.asset_path.clone().into(),
        info: animation.status.clone().into(),
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
            console: Default::default(),
            assets_activity: Default::default(),
            asset_browser: Default::default(),
            project_overview: Default::default(),
            module_plugins: Default::default(),
            build_export: Default::default(),
            ui_asset: Default::default(),
            animation: AnimationEditorPaneViewData {
                nodes: slint::ModelRc::default(),
                mode: "legacy".into(),
                asset_path: "legacy".into(),
                status: "legacy".into(),
                selection: "legacy".into(),
                current_frame: 0,
                timeline_start_frame: 0,
                timeline_end_frame: 0,
                playback_label: "legacy".into(),
                track_items: slint::ModelRc::default(),
                parameter_items: slint::ModelRc::default(),
                node_items: slint::ModelRc::default(),
                state_items: slint::ModelRc::default(),
                transition_items: slint::ModelRc::default(),
            },
        },
        pane_presentation: Some(pane_presentation),
    }
}

#[test]
fn animation_sequence_template_body_projects_hybrid_timeline_slot_for_slint_conversion() {
    let animation = animation_fixture("sequence");
    let projected = to_host_contract_animation_editor_pane_from_host_pane(
        &animation_pane(
            "pane.animation.sequence.body",
            PanePayloadKind::AnimationSequenceV1,
            PaneRouteNamespace::Animation,
            &animation,
        ),
        PaneContentSize::new(520.0, 300.0),
    );

    assert_eq!(projected.mode, "sequence");
    assert_eq!(projected.asset_path, "res://animations/hero.anim");
    assert_eq!(projected.current_frame, 12);
    assert_eq!(projected.timeline_start_frame, 5);
    assert_eq!(projected.timeline_end_frame, 42);
    assert_eq!(
        projected.track_items.row_data(0).as_deref(),
        Some("Root/Hero:Transform.position")
    );

    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert!(nodes.iter().any(|node| node.control_id == "ScrubTimeline"));
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "AnimationTimelineSlotAnchor"));
}

#[test]
fn animation_graph_template_body_projects_hybrid_canvas_slot_for_slint_conversion() {
    let animation = animation_fixture("graph");
    let projected = to_host_contract_animation_editor_pane_from_host_pane(
        &animation_pane(
            "pane.animation.graph.body",
            PanePayloadKind::AnimationGraphV1,
            PaneRouteNamespace::Animation,
            &animation,
        ),
        PaneContentSize::new(520.0, 300.0),
    );

    assert_eq!(projected.mode, "graph");
    assert_eq!(
        projected.parameter_items.row_data(0).as_deref(),
        Some("speed")
    );
    assert_eq!(projected.node_items.row_data(0).as_deref(), Some("Blend"));
    assert_eq!(projected.state_items.row_data(1).as_deref(), Some("Run"));
    assert_eq!(
        projected.transition_items.row_data(0).as_deref(),
        Some("Idle -> Run")
    );

    let nodes = (0..projected.nodes.row_count())
        .filter_map(|row| projected.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert!(nodes
        .iter()
        .any(|node| node.control_id == "AnimationGraphCanvasSlotAnchor"));
    assert!(nodes.iter().any(|node| node.control_id == "AddNode"));
}
