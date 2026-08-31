use crate::ui::retained_host::activity_rail_pointer::{
    HostActivityRailPointerBridge, HostActivityRailPointerRoute, HostActivityRailPointerSide,
};
use zircon_runtime_interface::ui::layout::UiPoint;

use super::support::sample_activity_rail_layout;

#[test]
fn shared_activity_rail_pointer_bridge_routes_left_and_right_button_hits() {
    let mut bridge = HostActivityRailPointerBridge::new();
    assert!(bridge.sync(sample_activity_rail_layout()));

    let left = bridge
        .handle_click(HostActivityRailPointerSide::Left, UiPoint::new(15.0, 20.0))
        .unwrap();
    assert_eq!(
        left.route,
        Some(HostActivityRailPointerRoute::Button {
            side: HostActivityRailPointerSide::Left,
            item_index: 0,
        })
    );
    assert_eq!(
        bridge.target_for_button(HostActivityRailPointerSide::Left, 0),
        Some(("left_top", "editor.project#1"))
    );

    let right = bridge
        .handle_click(HostActivityRailPointerSide::Right, UiPoint::new(15.0, 52.0))
        .unwrap();
    assert_eq!(
        right.route,
        Some(HostActivityRailPointerRoute::Button {
            side: HostActivityRailPointerSide::Right,
            item_index: 1,
        })
    );
    assert_eq!(
        bridge.target_for_button(HostActivityRailPointerSide::Right, 1),
        Some(("right_bottom", "editor.console#1"))
    );
}

#[test]
fn shared_activity_rail_pointer_bridge_accepts_projected_global_points() {
    let mut bridge = HostActivityRailPointerBridge::new();
    let layout = sample_activity_rail_layout();
    assert!(bridge.sync(layout.clone()));

    let left = bridge
        .handle_click_at_global_point(UiPoint::new(
            layout.left_strip_frame.x + 15.0,
            layout.left_strip_frame.y + 20.0,
        ))
        .unwrap();
    assert_eq!(
        left.route,
        Some(HostActivityRailPointerRoute::Button {
            side: HostActivityRailPointerSide::Left,
            item_index: 0,
        })
    );
}

#[test]
fn shared_activity_rail_pointer_bridge_skips_rebuild_for_unchanged_layout() {
    let mut bridge = HostActivityRailPointerBridge::new();
    let layout = sample_activity_rail_layout();

    assert!(bridge.sync(layout.clone()));
    assert!(!bridge.sync(layout));
}
