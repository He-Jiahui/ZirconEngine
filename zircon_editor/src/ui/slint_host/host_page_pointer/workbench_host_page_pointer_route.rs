#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchHostPagePointerRoute {
    Tab { item_index: usize, page_id: String },
}
