use zircon_runtime::ui::layout::UiFrame;

use super::host_document_tab_pointer_item::HostDocumentTabPointerItem;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostDocumentTabPointerSurface {
    pub key: String,
    pub strip_frame: UiFrame,
    pub items: Vec<HostDocumentTabPointerItem>,
}
