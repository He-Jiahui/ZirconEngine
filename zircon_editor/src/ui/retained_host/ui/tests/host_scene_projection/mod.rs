use super::*;
use std::sync::Arc;

mod assertions;
mod base_scene;
mod editor_panes;
mod workbench_panes;

#[test]
fn host_scene_projection_converts_host_owned_panes_to_host_contract_panes() {
    let mut scene = base_scene::host_scene();
    editor_panes::populate(&mut scene);
    workbench_panes::populate(&mut scene);

    let projected = to_host_contract_host_scene_data(&scene);
    assert_eq!(projected.page_chrome.overflow_widest_title_width_px, 123.0);
    assertions::assert_host_contract_scene(&projected);
}

#[test]
fn geometry_projection_reuses_pane_authority_while_updating_frames() {
    let scene = base_scene::host_scene();
    let mut projected = to_host_contract_host_scene_data(&scene);
    let left_frame = Arc::new(zircon_runtime_interface::ui::surface::UiSurfaceFrame::default());
    let document_frame = Arc::new(zircon_runtime_interface::ui::surface::UiSurfaceFrame::default());
    let right_frame = Arc::new(zircon_runtime_interface::ui::surface::UiSurfaceFrame::default());
    let bottom_frame = Arc::new(zircon_runtime_interface::ui::surface::UiSurfaceFrame::default());
    projected.left_dock.pane.body_surface_frame = Some(Arc::clone(&left_frame));
    projected.document_dock.pane.body_surface_frame = Some(Arc::clone(&document_frame));
    projected.right_dock.pane.body_surface_frame = Some(Arc::clone(&right_frame));
    projected.bottom_dock.pane.body_surface_frame = Some(Arc::clone(&bottom_frame));
    let mut resized_scene = scene.clone();
    resized_scene.layout.center_band_frame.width = 1600.0;
    resized_scene.left_dock.region_frame.width = 320.0;

    let resized =
        to_host_contract_host_scene_geometry_with_retained_panes(&resized_scene, &projected);

    assert_eq!(resized.layout.center_band_frame.width, 1600.0);
    assert_eq!(resized.left_dock.region_frame.width, 320.0);
    assert!(Arc::ptr_eq(
        &left_frame,
        resized.left_dock.pane.body_surface_frame.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        &document_frame,
        resized
            .document_dock
            .pane
            .body_surface_frame
            .as_ref()
            .unwrap()
    ));
    assert!(Arc::ptr_eq(
        &right_frame,
        resized.right_dock.pane.body_surface_frame.as_ref().unwrap()
    ));
    assert!(Arc::ptr_eq(
        &bottom_frame,
        resized
            .bottom_dock
            .pane
            .body_surface_frame
            .as_ref()
            .unwrap()
    ));
}
