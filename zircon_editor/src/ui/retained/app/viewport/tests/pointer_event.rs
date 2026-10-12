use super::*;
use crate::ui::retained_host::host_contract::{
    FrameRect, HostDocumentDockSurfaceData, HostWindowPresentationData,
};

#[test]
fn callback_surface_size_uses_the_source_window_leaf_extent() {
    let primary = scene_host("document:primary", 320.0, 200.0);
    let child = scene_host("native:child", 800.0, 600.0);

    assert_eq!(
        scene_viewport_surface_size(&primary, "document:primary"),
        Some(UiSize::new(320.0, 200.0))
    );
    let child_size = scene_viewport_surface_size(&child, "native:child").unwrap();
    assert_eq!(child_size, UiSize::new(800.0, 600.0));
    assert!(700.0 < child_size.width && 500.0 < child_size.height);
}

#[test]
fn viewport_surface_resolution_uses_the_live_scene_pane_id_and_preserves_game_panes() {
    let scene = viewport_host("surface:left", "editor.scene#left", "Scene");
    let game = viewport_host("surface:game", "editor.game#1", "Game");

    assert_eq!(
        scene_viewport_surface_target(&scene, "surface:left"),
        Some(SceneViewportSurfaceTarget::Scene(
            crate::core::editor_event::ViewInstanceId::new("editor.scene#left")
        ))
    );
    assert_eq!(
        scene_viewport_surface_target(&game, "surface:game"),
        Some(SceneViewportSurfaceTarget::OtherPane)
    );
    assert_eq!(
        scene_viewport_surface_target(&scene, "surface:retired"),
        None
    );
    assert_eq!(
        super::committed_scene_viewport_id_for_surface(&scene, "surface:left"),
        Some(crate::core::editor_event::ViewInstanceId::new(
            "editor.scene#left"
        ))
    );
    assert_eq!(
        super::committed_scene_viewport_id_for_surface(&game, "surface:game"),
        None
    );
}

fn scene_host(surface_key: &str, width: f32, height: f32) -> UiHostWindow {
    let ui = UiHostWindow::new().expect("viewport source host");
    let mut presentation = HostWindowPresentationData::default();
    let mut leaf = HostDocumentDockSurfaceData::default();
    leaf.surface_key = surface_key.into();
    leaf.content_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };
    presentation.host_scene_data.document_leaves.push(leaf);
    ui.set_host_presentation(presentation);
    ui
}

fn viewport_host(surface_key: &str, view_id: &str, kind: &str) -> UiHostWindow {
    let ui = UiHostWindow::new().expect("viewport source host");
    let mut presentation = HostWindowPresentationData::default();
    let mut leaf = HostDocumentDockSurfaceData::default();
    leaf.surface_key = surface_key.into();
    leaf.pane.id = view_id.into();
    leaf.pane.kind = kind.into();
    presentation.host_scene_data.document_leaves.push(leaf);
    ui.set_host_presentation(presentation);
    ui
}
