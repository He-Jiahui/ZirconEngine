use zircon_runtime::ui::layout::UiFrame;

use super::workbench_host_page_pointer_item::WorkbenchHostPagePointerItem;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorkbenchHostPagePointerLayout {
    pub strip_frame: UiFrame,
    pub items: Vec<WorkbenchHostPagePointerItem>,
}
