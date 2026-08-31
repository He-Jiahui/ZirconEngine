use super::host_drawer_header_pointer_item::HostDrawerHeaderPointerItem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HostDrawerHeaderPointerSurface {
    pub key: &'static str,
    pub items: Vec<HostDrawerHeaderPointerItem>,
}
