use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    text::{UiTextDocumentId, UiTextDocumentRevision},
};

// 同一节点编号可出现于不同树；会话绑定以树与节点共同识别编辑 owner。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct UiTextDocumentBindingKey {
    pub(super) tree_id: UiTreeId,
    pub(super) node_id: UiNodeId,
}

// 文档 revision 与表面 source_epoch 是两套一致性标记；编辑前须同时核对，避免使用尚未同步的表面源。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiTextDocumentBinding {
    pub(super) document_id: UiTextDocumentId,
    pub(super) revision: UiTextDocumentRevision,
    pub(super) source_epoch: u64,
}
