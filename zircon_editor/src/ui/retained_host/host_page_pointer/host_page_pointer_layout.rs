use super::host_page_pointer_item::HostPagePointerItem;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HostPagePointerLayout {
    pub items: Vec<HostPagePointerItem>,
}
