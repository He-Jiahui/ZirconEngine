#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchDocumentTabPointerRoute {
    ActivateTab {
        surface_key: String,
        item_index: usize,
        instance_id: String,
    },
    CloseTab {
        surface_key: String,
        item_index: usize,
        instance_id: String,
    },
}
