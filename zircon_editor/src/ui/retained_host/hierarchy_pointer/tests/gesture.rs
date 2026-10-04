use crate::ui::retained_host::hierarchy_pointer::{
    HierarchyPointerBridge, HierarchyPointerLayout, HierarchyPointerState,
};
use zircon_runtime_interface::ui::layout::UiPoint;

fn bridge() -> HierarchyPointerBridge {
    let mut bridge = HierarchyPointerBridge::new();
    bridge.sync(
        HierarchyPointerLayout {
            pane_width: 200.0,
            pane_height: 200.0,
            item_count: 3,
        },
        HierarchyPointerState::default(),
    );
    bridge
}

#[test]
fn hierarchy_reparent_cancel_and_release_are_terminal() {
    let mut bridge = bridge();
    bridge.begin_reparent_drag(UiPoint::new(80.0, 20.0));
    bridge.observe_reparent_drag_move(UiPoint::new(84.0, 20.0));
    bridge.cancel_reparent_drag();
    bridge.observe_reparent_drag_move(UiPoint::new(88.0, 20.0));
    assert!(!bridge.finish_reparent_drag());

    bridge.begin_reparent_drag(UiPoint::new(80.0, 20.0));
    bridge.observe_reparent_drag_move(UiPoint::new(84.0, 20.0));
    assert!(bridge.finish_reparent_drag());
    assert!(!bridge.finish_reparent_drag());
}

#[test]
fn hierarchy_reparent_invalid_origin_never_arms_a_gesture() {
    let mut bridge = bridge();
    for origin in [UiPoint::new(f32::NAN, 20.0), UiPoint::new(-1.0, 20.0)] {
        bridge.begin_reparent_drag(origin);
        bridge.observe_reparent_drag_move(UiPoint::new(84.0, 20.0));
        assert!(!bridge.finish_reparent_drag());
    }
}
