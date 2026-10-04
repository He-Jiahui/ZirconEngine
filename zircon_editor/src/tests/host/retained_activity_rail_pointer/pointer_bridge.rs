use crate::ui::retained_host::activity_rail_pointer::{
    HostActivityRailPointerBridge, HostActivityRailPointerItem, HostActivityRailPointerRoute,
    HostActivityRailPointerSide,
};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;
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
        bridge
            .target_for_button(HostActivityRailPointerSide::Left, 0)
            .map(|(slot, instance)| (slot, instance.clone())),
        Some((
            ActivityDrawerSlot::LeftTop,
            ViewInstanceId::new("editor.project#1")
        ))
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
        bridge
            .target_for_button(HostActivityRailPointerSide::Right, 1)
            .map(|(slot, instance)| (slot, instance.clone())),
        Some((
            ActivityDrawerSlot::RightBottom,
            ViewInstanceId::new("editor.console#1")
        ))
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

#[test]
fn shared_activity_rail_pointer_bridge_patches_geometry_without_rebuilding_authority() {
    let mut bridge = HostActivityRailPointerBridge::new();
    let mut layout = sample_activity_rail_layout();
    assert!(bridge.sync(layout.clone()));
    let authority_generation = bridge.surface_authority_generation_for_test();
    let old_button_point = UiPoint::new(15.0, 70.0);

    layout.left_strip_frame.y += 100.0;
    let moved_button_point = UiPoint::new(15.0, 170.0);
    assert!(bridge.sync(layout));

    assert_eq!(
        bridge.surface_authority_generation_for_test(),
        authority_generation,
        "geometry-only resize must retain Surface, dispatcher, and route authority"
    );
    assert_eq!(
        bridge
            .handle_click_at_global_point(old_button_point)
            .unwrap()
            .route,
        None
    );
    assert_eq!(
        bridge
            .handle_click_at_global_point(moved_button_point)
            .unwrap()
            .route,
        Some(HostActivityRailPointerRoute::Button {
            side: HostActivityRailPointerSide::Left,
            item_index: 0,
        })
    );
}

#[test]
fn shared_activity_rail_pointer_bridge_reuses_authority_for_equal_shape_semantics() {
    let mut bridge = HostActivityRailPointerBridge::new();
    let mut layout = sample_activity_rail_layout();
    assert!(bridge.sync(layout.clone()));
    let authority_generation = bridge.surface_authority_generation_for_test();

    let mut left_tabs = layout.left_tabs.as_ref().to_vec();
    left_tabs[0] = HostActivityRailPointerItem {
        slot: ActivityDrawerSlot::LeftTop,
        instance_id: ViewInstanceId::new("editor.replacement#1"),
    };
    layout.left_tabs = left_tabs.into();
    assert!(bridge.sync(layout));

    assert_eq!(
        bridge.surface_authority_generation_for_test(),
        authority_generation,
        "semantic target replacement must not rebuild unchanged hit topology"
    );
    assert_eq!(
        bridge
            .target_for_button(HostActivityRailPointerSide::Left, 0)
            .map(|(slot, instance)| (slot, instance.clone())),
        Some((
            ActivityDrawerSlot::LeftTop,
            ViewInstanceId::new("editor.replacement#1")
        ))
    );
}

#[test]
fn shared_activity_rail_pointer_bridge_rebuilds_when_visible_topology_changes() {
    let mut bridge = HostActivityRailPointerBridge::new();
    let mut layout = sample_activity_rail_layout();
    assert!(bridge.sync(layout.clone()));
    let authority_generation = bridge.surface_authority_generation_for_test();

    let mut left_tabs = layout.left_tabs.as_ref().to_vec();
    left_tabs.push(HostActivityRailPointerItem {
        slot: ActivityDrawerSlot::LeftBottom,
        instance_id: ViewInstanceId::new("editor.search#1"),
    });
    layout.left_tabs = left_tabs.into();
    assert!(bridge.sync(layout));

    assert!(
        bridge.surface_authority_generation_for_test() > authority_generation,
        "adding a visible button must rebuild Surface route topology"
    );
}
