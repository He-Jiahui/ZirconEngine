use super::*;
use crate::ui::layouts::common::model_rc;

#[test]
fn live_scene_value_patch_preserves_authored_toolbar_geometry_and_session_admission() {
    let mut pane = PaneData {
        kind: "Scene".into(),
        ..Default::default()
    };
    pane.viewport.toolbar_surface_key = "document:first".into();
    pane.viewport.toolbar_enter_play_enabled = true;
    pane.viewport.toolbar_template_nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ActivateSceneMode".into(),
        ..Default::default()
    }]);
    let next = SceneViewportChromeData {
        mode: "Transform.Move".into(),
        transform_space: "Global".into(),
        ..Default::default()
    };
    assert!(patch_scene_pane(&mut pane, &next));
    assert_eq!(pane.viewport.mode.as_str(), "Transform.Move");
    assert_eq!(pane.viewport.toolbar_template_nodes.row_count(), 1);
    assert_eq!(pane.viewport.toolbar_surface_key.as_str(), "document:first");
    assert!(pane.viewport.toolbar_enter_play_enabled);
}

#[test]
fn scene_chrome_patch_preserves_geometry_and_unrelated_rows() {
    let mut presentation = crate::ui::retained_host::HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane.kind = "Scene".to_string();
    let toolbar_frame = Arc::new(zircon_runtime_interface::ui::surface::UiSurfaceFrame::default());
    presentation
        .host_scene_data
        .document_dock
        .pane
        .viewport
        .toolbar_surface_frame = Some(Arc::clone(&toolbar_frame));
    presentation.workbench_window_nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: STATUS_GRID_CONTROL_ID.to_string(),
            text: "Grid: Off".to_string(),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "Unrelated".to_string(),
            text: "stable".to_string(),
            ..TemplatePaneNodeData::default()
        },
    ]);
    let mut state = HostContractState::new(
        crate::ui::retained_host::primitives::PhysicalSize::new(1280, 720),
    );
    state.replace_host_presentation(presentation);
    let original_nodes = state.host_presentation.workbench_window_nodes.clone();
    let viewport = SceneViewportChromeData {
        grid_mode: "VisibleAndSnap".to_string(),
        ..SceneViewportChromeData::default()
    };

    assert!(state.patch_scene_viewport_chrome(viewport, "Grid: 1 m", "Snap: On"));

    let patched = &state.host_presentation;
    assert_eq!(
        patched
            .host_scene_data
            .document_dock
            .pane
            .viewport
            .grid_mode,
        "VisibleAndSnap"
    );
    assert!(Arc::ptr_eq(
        patched
            .host_scene_data
            .document_dock
            .pane
            .viewport
            .toolbar_surface_frame
            .as_ref()
            .expect("toolbar frame"),
        &toolbar_frame,
    ));
    assert_eq!(
        patched.workbench_window_nodes.get(0).unwrap().text,
        "Grid: 1 m"
    );
    assert!(patched
        .workbench_window_nodes
        .shares_row_with(&original_nodes, 1));
}

#[test]
fn stable_scene_chrome_patch_keeps_the_retained_presentation() {
    let mut presentation = crate::ui::retained_host::HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane.kind = "Scene".to_string();
    presentation.host_scene_data.document_dock.pane.viewport = SceneViewportChromeData::default();
    presentation.workbench_window_nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: STATUS_GRID_CONTROL_ID.to_string(),
            text: "Grid: Off".to_string(),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: STATUS_SNAP_CONTROL_ID.to_string(),
            text: "Snap: Off".to_string(),
            ..TemplatePaneNodeData::default()
        },
    ]);
    let mut state = HostContractState::new(
        crate::ui::retained_host::primitives::PhysicalSize::new(1280, 720),
    );
    state.replace_host_presentation(presentation);
    let retained = Arc::clone(&state.host_presentation);

    assert!(!state.patch_scene_viewport_chrome(
        SceneViewportChromeData::default(),
        "Grid: Off",
        "Snap: Off",
    ));
    assert!(Arc::ptr_eq(&retained, &state.host_presentation));
}

#[test]
fn native_scene_chrome_patch_updates_only_the_cached_presenter_row() {
    let scene_window = FloatingWindowData {
        window_id: "window:scene".to_string(),
        active_pane: PaneData {
            kind: "Scene".to_string(),
            ..PaneData::default()
        },
        ..FloatingWindowData::default()
    };
    let hierarchy_window = FloatingWindowData {
        window_id: "window:hierarchy".to_string(),
        active_pane: PaneData {
            kind: "Hierarchy".to_string(),
            ..PaneData::default()
        },
        ..FloatingWindowData::default()
    };
    let mut presentation = crate::ui::retained_host::HostWindowPresentationData::default();
    presentation.native_floating_surface_data.floating_windows =
        model_rc(vec![scene_window, hierarchy_window]);
    let mut state = HostContractState::new(
        crate::ui::retained_host::primitives::PhysicalSize::new(1280, 720),
    );
    state.replace_host_presentation(presentation);
    let original = state
        .host_presentation
        .native_floating_surface_data
        .floating_windows
        .clone();
    let viewport = SceneViewportChromeData {
        mode: "Transform.Scale".to_string(),
        ..SceneViewportChromeData::default()
    };

    assert!(state.patch_native_scene_viewport_chrome(0, "window:scene", viewport.clone(),));
    let patched = &state
        .host_presentation
        .native_floating_surface_data
        .floating_windows;
    assert_eq!(
        patched.get(0).unwrap().active_pane.viewport.mode,
        "Transform.Scale"
    );
    assert!(patched.shares_row_with(&original, 1));
    assert!(!state.patch_native_scene_viewport_chrome(1, "window:scene", viewport.clone(),));
    assert!(!state.patch_native_scene_viewport_chrome(1, "window:hierarchy", viewport,));
}
