use zircon_runtime_interface::ui::layout::UiFrame;

use super::host_drawer_header_pointer_item::HostDrawerHeaderPointerItem;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostDrawerHeaderPointerSurface {
    pub key: String,
    pub strip_frame: UiFrame,
    pub items: Vec<HostDrawerHeaderPointerItem>,
}
