use crate::ui::retained_host::activity_rail_pointer::{
    HostActivityRailPointerItem, HostActivityRailPointerLayout,
};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) fn sample_activity_rail_layout() -> HostActivityRailPointerLayout {
    HostActivityRailPointerLayout {
        left_strip_frame: UiFrame::new(0.0, 51.0, 34.0, 400.0),
        left_tabs: vec![
            HostActivityRailPointerItem {
                slot: ActivityDrawerSlot::LeftTop,
                instance_id: ViewInstanceId::new("editor.project#1"),
            },
            HostActivityRailPointerItem {
                slot: ActivityDrawerSlot::LeftBottom,
                instance_id: ViewInstanceId::new("editor.hierarchy#1"),
            },
        ],
        right_strip_frame: UiFrame::new(1246.0, 51.0, 34.0, 400.0),
        right_tabs: vec![
            HostActivityRailPointerItem {
                slot: ActivityDrawerSlot::RightTop,
                instance_id: ViewInstanceId::new("editor.inspector#1"),
            },
            HostActivityRailPointerItem {
                slot: ActivityDrawerSlot::RightBottom,
                instance_id: ViewInstanceId::new("editor.console#1"),
            },
        ],
    }
}
