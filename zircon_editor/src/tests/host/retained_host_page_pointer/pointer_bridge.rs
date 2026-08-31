use crate::ui::retained_host::host_page_pointer::{HostPagePointerBridge, HostPagePointerRoute};

use super::support::sample_host_page_layout;

#[test]
fn shared_host_page_pointer_bridge_routes_native_activation_receipt() {
    let mut bridge = HostPagePointerBridge::new();
    assert!(bridge.sync(sample_host_page_layout()));

    let dispatch = bridge.handle_click(1, false).unwrap();
    let route = dispatch.route.expect("activation route");
    assert_eq!(route, HostPagePointerRoute::Activate { item_index: 1 });
    assert_eq!(
        bridge
            .activation_target_for_route(route)
            .expect("activation target")
            .0,
        "inspector"
    );
}

#[test]
fn shared_host_page_pointer_bridge_routes_native_close_receipt() {
    let mut bridge = HostPagePointerBridge::new();
    assert!(bridge.sync(sample_host_page_layout()));

    let dispatch = bridge.handle_click(1, true).unwrap();
    let route = dispatch.route.expect("close route");
    assert_eq!(route, HostPagePointerRoute::Close { item_index: 1 });
    assert_eq!(
        bridge
            .close_target_for_route(route)
            .expect("close target")
            .0,
        "editor.prefab#1"
    );
}

#[test]
fn shared_host_page_pointer_bridge_rejects_noncloseable_and_stale_receipts() {
    let mut bridge = HostPagePointerBridge::new();
    assert!(bridge.sync(sample_host_page_layout()));

    assert!(bridge.handle_click(0, true).is_err());
    assert!(bridge.handle_click(99, false).is_err());
}

#[test]
fn shared_host_page_pointer_bridge_skips_unchanged_receipt_projection() {
    let mut bridge = HostPagePointerBridge::new();
    let layout = sample_host_page_layout();

    assert!(bridge.sync(layout.clone()));
    assert!(!bridge.sync(layout));
}
