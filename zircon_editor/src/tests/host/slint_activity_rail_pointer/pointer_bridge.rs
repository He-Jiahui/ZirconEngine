use crate::ui::slint_host::activity_rail_pointer::{
    WorkbenchActivityRailPointerBridge, WorkbenchActivityRailPointerRoute,
    WorkbenchActivityRailPointerSide,
};
use zircon_runtime::ui::layout::UiPoint;

use super::support::sample_activity_rail_layout;

#[test]
fn shared_activity_rail_pointer_bridge_routes_left_and_right_button_hits() {
    let mut bridge = WorkbenchActivityRailPointerBridge::new();
    bridge.sync(sample_activity_rail_layout());

    let left = bridge
        .handle_click(
            WorkbenchActivityRailPointerSide::Left,
            UiPoint::new(15.0, 20.0),
        )
        .unwrap();
    assert_eq!(
        left.route,
        Some(WorkbenchActivityRailPointerRoute::Button {
            side: WorkbenchActivityRailPointerSide::Left,
            item_index: 0,
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        })
    );

    let right = bridge
        .handle_click(
            WorkbenchActivityRailPointerSide::Right,
            UiPoint::new(15.0, 52.0),
        )
        .unwrap();
    assert_eq!(
        right.route,
        Some(WorkbenchActivityRailPointerRoute::Button {
            side: WorkbenchActivityRailPointerSide::Right,
            item_index: 1,
            slot: "right_bottom".to_string(),
            instance_id: "editor.console#1".to_string(),
        })
    );
}

#[test]
fn shared_activity_rail_pointer_bridge_accepts_projected_global_points() {
    let mut bridge = WorkbenchActivityRailPointerBridge::new();
    let layout = sample_activity_rail_layout();
    bridge.sync(layout.clone());

    let left = bridge
        .handle_click(
            WorkbenchActivityRailPointerSide::Left,
            UiPoint::new(
                layout.left_strip_frame.x + 15.0,
                layout.left_strip_frame.y + 20.0,
            ),
        )
        .unwrap();
    assert_eq!(
        left.route,
        Some(WorkbenchActivityRailPointerRoute::Button {
            side: WorkbenchActivityRailPointerSide::Left,
            item_index: 0,
            slot: "left_top".to_string(),
            instance_id: "editor.project#1".to_string(),
        })
    );
}
