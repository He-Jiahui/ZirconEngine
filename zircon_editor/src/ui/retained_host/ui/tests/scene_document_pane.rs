use super::*;

#[test]
fn scene_document_pane_uses_viewport_dimensions_and_enables_toolbar() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let ui_asset_panes = BTreeMap::new();

    let pane = document_pane(&model, &chrome, &ui_asset_panes, &BTreeMap::new(), None);

    assert_eq!(pane.kind, "Scene");
    assert_eq!(pane.subtitle, "1280 x 720");
    assert!(pane.show_toolbar);
}

#[test]
fn scene_document_pane_projects_viewport_toolbar_state() {
    let mut fixture = default_preview_fixture();
    fixture.editor.scene_viewport_settings.mode =
        SceneModeActivation::Transform(TransformHandleKind::Scale);
    fixture.editor.scene_viewport_settings.transform_space = TransformSpace::Global;
    fixture.editor.scene_viewport_settings.projection_mode = ProjectionMode::Orthographic;
    fixture.editor.scene_viewport_settings.view_orientation = ViewOrientation::NegZ;
    fixture.editor.scene_viewport_settings.display_mode = DisplayMode::WireOverlay;
    fixture.editor.scene_viewport_settings.grid_mode = GridMode::VisibleAndSnap;
    fixture.editor.scene_viewport_settings.gizmos_enabled = false;
    fixture.editor.scene_viewport_settings.preview_lighting = false;
    fixture.editor.scene_viewport_settings.preview_skybox = false;
    fixture.editor.scene_viewport_settings.translate_step = 2.5;
    fixture.editor.scene_viewport_settings.rotate_step_deg = 30.0;
    fixture.editor.scene_viewport_settings.scale_step = 0.25;

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let ui_asset_panes = BTreeMap::new();
    let pane = document_pane(&model, &chrome, &ui_asset_panes, &BTreeMap::new(), None);

    assert_eq!(pane.kind, "Scene");
    assert_eq!(pane.viewport.mode, "Transform.Scale");
    assert_eq!(pane.viewport.transform_space, "Global");
    assert_eq!(pane.viewport.projection_mode, "Orthographic");
    assert_eq!(pane.viewport.view_orientation, "NegZ");
    assert_eq!(pane.viewport.display_mode, "WireOverlay");
    assert_eq!(pane.viewport.grid_mode, "VisibleAndSnap");
    assert!(!pane.viewport.gizmos_enabled);
    assert!(!pane.viewport.preview_lighting);
    assert!(!pane.viewport.preview_skybox);
    assert_eq!(pane.viewport.translate_snap, 2.5);
    assert_eq!(pane.viewport.rotate_snap_deg, 30.0);
    assert_eq!(pane.viewport.scale_snap, 0.25);
    assert_eq!(pane.viewport.translate_snap_label, "T 2.5");
    assert_eq!(pane.viewport.rotate_snap_label, "R 30");
    assert_eq!(pane.viewport.scale_snap_label, "S 0.25");
}
