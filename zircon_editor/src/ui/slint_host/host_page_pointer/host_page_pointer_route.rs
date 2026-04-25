#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HostPagePointerRoute {
    Tab { item_index: usize, page_id: String },
}
