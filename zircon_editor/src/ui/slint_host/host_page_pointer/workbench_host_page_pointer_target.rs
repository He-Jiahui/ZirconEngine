#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum WorkbenchHostPagePointerTarget {
    Tab { item_index: usize, page_id: String },
}
