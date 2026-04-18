use zircon_ui::UiFrame;

use super::workbench_document_tab_pointer_item::WorkbenchDocumentTabPointerItem;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchDocumentTabPointerSurface {
    pub key: String,
    pub strip_frame: UiFrame,
    pub items: Vec<WorkbenchDocumentTabPointerItem>,
}
