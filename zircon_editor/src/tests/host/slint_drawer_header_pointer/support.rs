use crate::ui::slint_host::drawer_header_pointer::{
    HostDrawerHeaderPointerItem, HostDrawerHeaderPointerLayout, HostDrawerHeaderPointerSurface,
};
use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) fn sample_drawer_header_layout() -> HostDrawerHeaderPointerLayout {
    HostDrawerHeaderPointerLayout {
        surfaces: vec![
            HostDrawerHeaderPointerSurface {
                key: "left".to_string(),
                strip_frame: UiFrame::new(35.0, 53.0, 240.0, 25.0),
                items: vec![
                    HostDrawerHeaderPointerItem {
                        slot: "left_top".to_string(),
                        instance_id: "editor.project#1".to_string(),
                    },
                    HostDrawerHeaderPointerItem {
                        slot: "left_bottom".to_string(),
                        instance_id: "editor.hierarchy#1".to_string(),
                    },
                ],
            },
            HostDrawerHeaderPointerSurface {
                key: "right".to_string(),
                strip_frame: UiFrame::new(1002.0, 53.0, 240.0, 25.0),
                items: vec![HostDrawerHeaderPointerItem {
                    slot: "right_top".to_string(),
                    instance_id: "editor.inspector#1".to_string(),
                }],
            },
        ],
    }
}
