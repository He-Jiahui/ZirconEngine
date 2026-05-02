use crate::ui::slint_host::viewport_toolbar_pointer::{
    build_viewport_toolbar_pointer_layout, ViewportToolbarPointerBridge,
    ViewportToolbarPointerRoute,
};
use zircon_runtime_interface::ui::layout::UiPoint;

#[test]
fn shared_viewport_toolbar_pointer_bridge_routes_controls_from_shared_hit_test() {
    let mut bridge = ViewportToolbarPointerBridge::new();
    bridge.sync(build_viewport_toolbar_pointer_layout(["scene.main"]));

    let route = bridge
        .handle_click(
            "scene.main",
            "tool.scale",
            120.0,
            0.0,
            40.0,
            20.0,
            UiPoint::new(132.0, 10.0),
        )
        .unwrap();
    assert_eq!(
        route.route,
        Some(ViewportToolbarPointerRoute::SetTool {
            surface_key: "scene.main".to_string(),
            tool: "Scale".to_string(),
        })
    );
}
