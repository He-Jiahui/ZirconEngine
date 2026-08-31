use crate::ui::retained_host::drawer_header_pointer::{
    HostDrawerHeaderPointerBridge, HostDrawerHeaderPointerRoute,
};
use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::support::sample_drawer_header_layout;

#[test]
fn shared_drawer_header_pointer_bridge_routes_group_tabs_from_shared_hit_test() {
    let mut bridge = HostDrawerHeaderPointerBridge::new();
    assert!(bridge.sync(sample_drawer_header_layout()));

    let route = bridge.handle_click("left", 1).unwrap();
    assert_eq!(
        route.route,
        Some(HostDrawerHeaderPointerRoute::Tab {
            surface_index: 0,
            item_index: 1,
        })
    );
    assert_eq!(
        bridge
            .target_for_route(route.route.expect("drawer header route"))
            .map(|(slot, instance_id)| (slot, instance_id.0.as_str())),
        Some((ActivityDrawerSlot::LeftBottom, "editor.hierarchy#1"))
    );
}

#[test]
fn shared_drawer_header_pointer_bridge_skips_rebuild_for_unchanged_layout() {
    let mut bridge = HostDrawerHeaderPointerBridge::new();
    let layout = sample_drawer_header_layout();

    assert!(bridge.sync(layout.clone()));
    assert!(!bridge.sync(layout));
}

#[test]
fn shared_drawer_header_native_receipt_rejects_unknown_surface_and_index() {
    let mut bridge = HostDrawerHeaderPointerBridge::new();
    assert!(bridge.sync(sample_drawer_header_layout()));

    assert!(bridge.handle_click("left", 99).is_err());
    assert!(bridge.handle_click("missing", 0).is_err());
}
