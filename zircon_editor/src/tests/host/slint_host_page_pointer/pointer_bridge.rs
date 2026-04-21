use crate::ui::slint_host::host_page_pointer::{
    WorkbenchHostPagePointerBridge, WorkbenchHostPagePointerRoute,
};
use zircon_runtime::ui::layout::UiPoint;

use super::support::sample_host_page_layout;

#[test]
fn shared_host_page_pointer_bridge_routes_tabs_from_shared_hit_test() {
    let mut bridge = WorkbenchHostPagePointerBridge::new();
    bridge.sync(sample_host_page_layout());

    let route = bridge
        .handle_click(1, 80.0, 92.0, UiPoint::new(90.0, 12.0))
        .unwrap();
    assert_eq!(
        route.route,
        Some(WorkbenchHostPagePointerRoute::Tab {
            item_index: 1,
            page_id: "inspector".to_string(),
        })
    );
}
