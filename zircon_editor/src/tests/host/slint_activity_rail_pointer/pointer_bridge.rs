use crate::ui::slint_host::activity_rail_pointer::{
    HostActivityRailPointerBridge, HostActivityRailPointerRoute, HostActivityRailPointerSide,
};
use zircon_runtime::ui::layout::UiPoint;

use super::support::sample_activity_rail_layout;

#[test]
fn shared_activity_rail_pointer_bridge_routes_left_and_right_button_hits() {
    let mut bridge = HostActivityRailPointerBridge::new();
    bridge.sync(sample_activity_rail_layout());

    let left = bridge
        .handle_click(HostActivityRailPointerSide::Left, UiPoint::new(15.0, 20.0))
        .unwrap();
    assert_eq!(
        left.route,
        Some(HostActivityRailPointerRoute::Button {
            side: HostActivityRailPointerSide::Left,
            item_index: 0,
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        })
    );

    let right = bridge
        .handle_click(HostActivityRailPointerSide::Right, UiPoint::new(15.0, 52.0))
        .unwrap();
    assert_eq!(
        right.route,
        Some(HostActivityRailPointerRoute::Button {
            side: HostActivityRailPointerSide::Right,
            item_index: 1,
            slot: "right_bottom".to_string(),
            instance_id: "editor.console#1".to_string(),
        })
    );
}

#[test]
fn shared_activity_rail_pointer_bridge_accepts_projected_global_points() {
    let mut bridge = HostActivityRailPointerBridge::new();
    let layout = sample_activity_rail_layout();
    bridge.sync(layout.clone());

    let left = bridge
        .handle_click(
            HostActivityRailPointerSide::Left,
            UiPoint::new(
                layout.left_strip_frame.x + 15.0,
                layout.left_strip_frame.y + 20.0,
            ),
        )
        .unwrap();
    assert_eq!(
        left.route,
        Some(HostActivityRailPointerRoute::Button {
            side: HostActivityRailPointerSide::Left,
            item_index: 0,
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        })
    );
}
