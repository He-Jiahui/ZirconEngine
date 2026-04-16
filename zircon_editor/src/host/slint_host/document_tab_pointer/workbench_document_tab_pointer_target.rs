#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::host::slint_host::document_tab_pointer) enum WorkbenchDocumentTabPointerTarget {
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
