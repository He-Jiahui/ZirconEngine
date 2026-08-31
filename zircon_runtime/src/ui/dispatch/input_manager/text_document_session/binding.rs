use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    text::{UiTextDocumentId, UiTextDocumentRevision},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct UiTextDocumentBindingKey {
    pub(super) tree_id: UiTreeId,
    pub(super) node_id: UiNodeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiTextDocumentBinding {
    pub(super) document_id: UiTextDocumentId,
    pub(super) revision: UiTextDocumentRevision,
    pub(super) source_epoch: u64,
}
