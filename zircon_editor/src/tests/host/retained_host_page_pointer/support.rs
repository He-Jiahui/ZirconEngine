use crate::core::editor_event::MainPageId;
use crate::ui::retained_host::host_page_pointer::{
    HostPageOverflowSlot, HostPagePointerItem, HostPagePointerLayout, HostPageTabSlot,
};
use crate::ui::workbench::page_tabs::main_page_tab_close_frame;
use zircon_runtime_interface::ui::layout::UiFrame;

pub(super) fn sample_host_page_layout() -> HostPagePointerLayout {
    let items = vec![
        HostPagePointerItem {
            page_id: MainPageId::workbench().0,
            title: "Workbench".to_string(),
            close_instance_id: None,
        },
        HostPagePointerItem {
            page_id: "inspector".to_string(),
            title: "Inspector".to_string(),
            close_instance_id: Some("editor.prefab#1".to_string()),
        },
    ];
    let strip_frame = UiFrame::new(0.0, 24.0, 1280.0, 32.0);
    let tabs = items
        .iter()
        .enumerate()
        .map(|(page_index, item)| {
            let frame = UiFrame::new(8.0 + page_index as f32 * 112.0, 25.0, 108.0, 30.0);
            HostPageTabSlot {
                page_index,
                page_id: item.page_id.clone(),
                close_frame: item
                    .close_instance_id
                    .as_ref()
                    .map(|_| main_page_tab_close_frame(frame)),
                frame,
            }
        })
        .collect();
    HostPagePointerLayout {
        strip_frame,
        items,
        tabs,
        overflow: None,
    }
}

pub(super) fn sample_overflow_host_page_layout() -> HostPagePointerLayout {
    let items = vec![
        HostPagePointerItem {
            page_id: MainPageId::workbench().0,
            title: "Workbench".to_string(),
            close_instance_id: None,
        },
        HostPagePointerItem {
            page_id: "inspector".to_string(),
            title: "Inspector".to_string(),
            close_instance_id: None,
        },
        HostPagePointerItem {
            page_id: "assets".to_string(),
            title: "Assets".to_string(),
            close_instance_id: None,
        },
        HostPagePointerItem {
            page_id: "animation".to_string(),
            title: "Animation".to_string(),
            close_instance_id: None,
        },
    ];
    let strip_frame = UiFrame::new(0.0, 24.0, 360.0, 32.0);
    HostPagePointerLayout {
        strip_frame,
        items: items.clone(),
        tabs: vec![
            HostPageTabSlot {
                page_index: 0,
                page_id: items[0].page_id.clone(),
                frame: UiFrame::new(8.0, 25.0, 108.0, 30.0),
                close_frame: None,
            },
            HostPageTabSlot {
                page_index: 1,
                page_id: items[1].page_id.clone(),
                frame: UiFrame::new(122.0, 25.0, 108.0, 30.0),
                close_frame: None,
            },
        ],
        overflow: Some(HostPageOverflowSlot {
            frame: UiFrame::new(236.0, 25.0, 34.0, 30.0),
            hidden_page_indices: vec![2, 3],
        }),
    }
}
