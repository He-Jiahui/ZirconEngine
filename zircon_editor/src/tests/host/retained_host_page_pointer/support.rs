use crate::ui::retained_host::host_page_pointer::{HostPagePointerItem, HostPagePointerLayout};
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

pub(super) fn sample_host_page_layout() -> HostPagePointerLayout {
    HostPagePointerLayout {
        items: vec![
            HostPagePointerItem {
                page_id: MainPageId::workbench(),
                close_instance_id: None,
            },
            HostPagePointerItem {
                page_id: MainPageId::new("inspector"),
                close_instance_id: Some(ViewInstanceId::new("editor.prefab#1")),
            },
        ],
    }
}

pub(super) fn sample_overflow_host_page_layout() -> HostPagePointerLayout {
    HostPagePointerLayout {
        items: ["workbench", "inspector", "assets", "animation"]
            .into_iter()
            .map(|page_id| HostPagePointerItem {
                page_id: MainPageId::new(page_id),
                close_instance_id: None,
            })
            .collect(),
    }
}
