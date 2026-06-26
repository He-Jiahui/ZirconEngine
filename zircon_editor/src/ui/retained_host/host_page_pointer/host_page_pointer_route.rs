#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HostPagePointerRoute {
    Tab { item_index: usize, page_id: String },
    Overflow { hidden_page_indices: Vec<usize> },
}
