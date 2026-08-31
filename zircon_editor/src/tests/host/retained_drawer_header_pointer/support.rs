use crate::ui::retained_host::drawer_header_pointer::{
    HostDrawerHeaderPointerItem, HostDrawerHeaderPointerLayout, HostDrawerHeaderPointerSurface,
};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;

pub(super) fn sample_drawer_header_layout() -> HostDrawerHeaderPointerLayout {
    HostDrawerHeaderPointerLayout {
        surfaces: vec![
            HostDrawerHeaderPointerSurface {
                key: "left",
                items: vec![
                    HostDrawerHeaderPointerItem {
                        slot: ActivityDrawerSlot::LeftTop,
                        instance_id: ViewInstanceId::new("editor.project#1"),
                    },
                    HostDrawerHeaderPointerItem {
                        slot: ActivityDrawerSlot::LeftBottom,
                        instance_id: ViewInstanceId::new("editor.hierarchy#1"),
                    },
                ],
            },
            HostDrawerHeaderPointerSurface {
                key: "right",
                items: vec![HostDrawerHeaderPointerItem {
                    slot: ActivityDrawerSlot::RightTop,
                    instance_id: ViewInstanceId::new("editor.inspector#1"),
                }],
            },
        ],
    }
}
