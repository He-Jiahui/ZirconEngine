use zircon_runtime_interface::ui::layout::UiFrame;

use super::host_page_pointer_item::HostPagePointerItem;
use super::tab_strip_geometry::{HostPageOverflowSlot, HostPageTabSlot};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HostPagePointerLayout {
    pub strip_frame: UiFrame,
    pub items: Vec<HostPagePointerItem>,
    pub tabs: Vec<HostPageTabSlot>,
    pub overflow: Option<HostPageOverflowSlot>,
}
