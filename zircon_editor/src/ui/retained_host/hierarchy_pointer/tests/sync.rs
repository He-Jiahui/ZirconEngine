use super::*;
use crate::ui::retained_host::hierarchy_pointer::HierarchyRowMetrics;

#[test]
fn authored_row_geometry_change_clamps_and_publishes_native_scroll_state() {
    let mut bridge = HierarchyPointerBridge::default();
    let large = HierarchyRowMetrics {
        row_x: 0.0,
        row_y: 0.0,
        row_height: 40.0,
        row_gap: 0.0,
        row_width_inset: 0.0,
    };
    bridge.set_authored_row_metrics(Some(large));
    bridge.sync(
        HierarchyPointerLayout {
            pane_width: 240.0,
            pane_height: 100.0,
            item_count: 10,
        },
        HierarchyPointerState {
            scroll_offset: 200.0,
            ..Default::default()
        },
    );
    let compact = HierarchyRowMetrics {
        row_height: 10.0,
        ..large
    };
    assert!(bridge.set_authored_row_metrics(Some(compact)));
    assert_eq!(bridge.committed_state().scroll_offset, 0.0);
    assert_eq!(bridge.row_metrics, compact);
    assert!(!bridge.set_authored_row_metrics(Some(compact)));
}
