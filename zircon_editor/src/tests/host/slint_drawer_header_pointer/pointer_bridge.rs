use crate::ui::slint_host::drawer_header_pointer::{
    HostDrawerHeaderPointerBridge, HostDrawerHeaderPointerRoute,
};
use zircon_runtime_interface::ui::layout::UiPoint;

use super::support::sample_drawer_header_layout;

#[test]
fn shared_drawer_header_pointer_bridge_routes_group_tabs_from_shared_hit_test() {
    let mut bridge = HostDrawerHeaderPointerBridge::new();
    bridge.sync(sample_drawer_header_layout());

    let route = bridge
        .handle_click("left", 1, 112.0, 96.0, UiPoint::new(120.0, 12.0))
        .unwrap();
    assert_eq!(
        route.route,
        Some(HostDrawerHeaderPointerRoute::Tab {
            surface_key: "left".to_string(),
            item_index: 1,
            slot: "left_bottom".to_string(),
            instance_id: "editor.hierarchy#1".to_string(),
        })
    );
}
