use crate::core::editor_event::MainPageId;
use crate::ui::slint_host::host_page_pointer::{HostPagePointerItem, HostPagePointerLayout};
use zircon_runtime::ui::layout::UiFrame;

pub(super) fn sample_host_page_layout() -> HostPagePointerLayout {
    HostPagePointerLayout {
        strip_frame: UiFrame::new(0.0, 26.0, 1280.0, 32.0),
        items: vec![
            HostPagePointerItem {
                page_id: MainPageId::workbench().0,
            },
            HostPagePointerItem {
                page_id: "inspector".to_string(),
            },
        ],
    }
}
