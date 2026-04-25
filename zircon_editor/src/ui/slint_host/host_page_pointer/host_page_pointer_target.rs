#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum HostPagePointerTarget {
    Tab { item_index: usize, page_id: String },
}
