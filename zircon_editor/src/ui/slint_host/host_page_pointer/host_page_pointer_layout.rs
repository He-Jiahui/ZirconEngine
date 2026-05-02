use zircon_runtime_interface::ui::layout::UiFrame;

use super::host_page_pointer_item::HostPagePointerItem;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HostPagePointerLayout {
    pub strip_frame: UiFrame,
    pub items: Vec<HostPagePointerItem>,
}
