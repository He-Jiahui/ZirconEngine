use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

use super::*;

#[test]
fn hit_test_viewport_toolbar_uses_shared_surface_frame_node_geometry() {
    let toolbar = FrameRect {
        x: 100.0,
        y: 200.0,
        width: 640.0,
        height: 28.0,
    };
    let viewport = SceneViewportChromeData {
        toolbar_surface_frame: Some(surface_frame_for_test(
            "custom.button",
            UiFrame::new(23.0, 4.0, 77.0, 20.0),
        )),
        ..SceneViewportChromeData::default()
    };

    let hit = hit_test_viewport_toolbar("scene.main", &viewport, &toolbar, 140.0, 214.0)
        .expect("point inside shared arranged button should hit");

    assert_eq!(hit.surface_key.as_str(), "scene.main");
    assert_eq!(hit.control_id.as_str(), "custom.button");
    assert_eq!(hit.control_x, 23.0);
    assert_eq!(hit.control_y, 4.0);
    assert_eq!(hit.control_width, 77.0);
    assert_eq!(hit.control_height, 20.0);
}

#[test]
fn hit_test_viewport_toolbar_returns_none_without_surface_frame() {
    let toolbar = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 640.0,
        height: 28.0,
    };
    assert!(hit_test_viewport_toolbar(
        "scene.main",
        &SceneViewportChromeData::default(),
        &toolbar,
        32.0,
        12.0,
    )
    .is_none());
}

fn surface_frame_for_test(
    control_id: &str,
    frame: UiFrame,
) -> zircon_runtime_interface::ui::surface::UiSurfaceFrame {
    let mut surface = UiSurface::new(UiTreeId::new("test.viewport_toolbar"));
    let root_frame = UiFrame::new(0.0, 0.0, 640.0, 28.0);
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);

    let node = UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
        .with_frame(frame)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: true,
            pressed: false,
            checked: false,
            dirty: false,
        })
        .with_input_policy(UiInputPolicy::Receive)
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "IconButton".to_string(),
            control_id: Some(control_id.to_string()),
            ..Default::default()
        });
    surface.tree.insert_child(UiNodeId::new(1), node).unwrap();
    surface.rebuild();
    surface.surface_frame()
}
