use crate::ui::slint_host::activity_rail_pointer::{
    HostActivityRailPointerItem, HostActivityRailPointerLayout,
};
use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) fn sample_activity_rail_layout() -> HostActivityRailPointerLayout {
    HostActivityRailPointerLayout {
        left_strip_frame: UiFrame::new(0.0, 51.0, 34.0, 400.0),
        left_tabs: vec![
            HostActivityRailPointerItem {
                slot: "left_top".to_string(),
                instance_id: "editor.project#1".to_string(),
            },
            HostActivityRailPointerItem {
                slot: "left_bottom".to_string(),
                instance_id: "editor.hierarchy#1".to_string(),
            },
        ],
        right_strip_frame: UiFrame::new(1246.0, 51.0, 34.0, 400.0),
        right_tabs: vec![
            HostActivityRailPointerItem {
                slot: "right_top".to_string(),
                instance_id: "editor.inspector#1".to_string(),
            },
            HostActivityRailPointerItem {
                slot: "right_bottom".to_string(),
                instance_id: "editor.console#1".to_string(),
            },
        ],
    }
}
