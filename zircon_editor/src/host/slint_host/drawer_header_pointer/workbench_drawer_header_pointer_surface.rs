use zircon_ui::UiFrame;

use super::workbench_drawer_header_pointer_item::WorkbenchDrawerHeaderPointerItem;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchDrawerHeaderPointerSurface {
    pub key: String,
    pub strip_frame: UiFrame,
    pub items: Vec<WorkbenchDrawerHeaderPointerItem>,
}
