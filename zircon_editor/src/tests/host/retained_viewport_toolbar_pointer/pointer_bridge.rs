use crate::ui::retained_host::viewport_toolbar_pointer::{
    ViewportToolbarPointerBridge, ViewportToolbarPointerRoute,
    build_viewport_toolbar_pointer_layout,
};
use zircon_runtime_interface::ui::layout::UiPoint;

#[test]
fn shared_viewport_toolbar_pointer_bridge_routes_controls_from_shared_hit_test() {
    let mut bridge = ViewportToolbarPointerBridge::new();
    assert!(bridge.sync(build_viewport_toolbar_pointer_layout(["scene.main"])));

    let route = bridge
        .handle_click(
            "scene.main",
            "mode.scale",
            120.0,
            0.0,
            40.0,
            20.0,
            UiPoint::new(132.0, 10.0),
        )
        .unwrap();
    assert_eq!(
        route.route,
        Some(ViewportToolbarPointerRoute::ActivateSceneMode {
            surface_key: "scene.main".to_string(),
            mode: "Transform.Scale".to_string(),
        })
    );

    let play = bridge
        .handle_click(
            "scene.main",
            "EnterPlayMode",
            684.0,
            0.0,
            24.0,
            20.0,
            UiPoint::new(692.0, 10.0),
        )
        .unwrap();
    assert_eq!(
        play.route,
        Some(ViewportToolbarPointerRoute::EnterPlayMode {
            surface_key: "scene.main".to_string(),
        })
    );

    let scale_again = bridge
        .handle_click_at_point("scene.main", UiPoint::new(132.0, 10.0))
        .unwrap();
    assert_eq!(
        scale_again.route,
        Some(ViewportToolbarPointerRoute::ActivateSceneMode {
            surface_key: "scene.main".to_string(),
            mode: "Transform.Scale".to_string(),
        }),
        "syncing one clicked control must not discard the other committed toolbar controls"
    );
}

#[test]
fn shared_viewport_toolbar_pointer_bridge_skips_rebuild_for_unchanged_layout() {
    let mut bridge = ViewportToolbarPointerBridge::new();
    let layout = build_viewport_toolbar_pointer_layout(["scene.main"]);

    assert!(bridge.sync(layout.clone()));
    assert!(!bridge.sync(layout));
}
