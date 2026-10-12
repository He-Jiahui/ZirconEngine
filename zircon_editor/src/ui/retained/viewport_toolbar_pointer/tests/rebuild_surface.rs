use super::*;
use crate::ui::retained_host::viewport_toolbar_pointer::{
    viewport_toolbar_pointer_control::ViewportToolbarPointerControl,
    viewport_toolbar_pointer_surface::ViewportToolbarPointerSurface,
};

#[test]
fn stable_toolbar_control_frame_uses_exact_runtime_geometry_publication() {
    let mut bridge = ViewportToolbarPointerBridge::new();
    bridge.layout.surfaces = vec![ViewportToolbarPointerSurface {
        key: "scene.main".to_string(),
        frame: UiFrame::new(0.0, 0.0, 320.0, 40.0),
    }];
    bridge.controls_by_surface.insert(
        "scene.main".to_string(),
        vec![ViewportToolbarPointerControl {
            action_key: "mode.select".to_string(),
            frame: UiFrame::new(8.0, 8.0, 24.0, 24.0),
        }],
    );
    bridge.rebuild_surface_from_scratch();

    bridge
        .controls_by_surface
        .get_mut("scene.main")
        .expect("toolbar controls must exist")[0]
        .frame = UiFrame::new(40.0, 8.0, 24.0, 24.0);
    bridge.apply_surface_delta(ViewportToolbarSurfaceDelta::Geometry(vec![
        ViewportToolbarNodeFrameChange {
            node_id: viewport_toolbar_control_node_id(0, 0),
            frame: UiFrame::new(40.0, 8.0, 24.0, 24.0),
        },
    ]));

    assert_eq!(
        bridge
            .surface
            .last_rebuild_report
            .arranged_outer_node_visit_count,
        1
    );
    assert_eq!(
        bridge
            .surface
            .last_rebuild_report
            .hit_grid_outer_node_visit_count,
        1
    );
    assert_eq!(
        bridge
            .surface
            .last_rebuild_report
            .render_outer_node_visit_count,
        1
    );
}
