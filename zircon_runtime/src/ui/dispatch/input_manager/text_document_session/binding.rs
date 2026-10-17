use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    text::{UiTextDocumentId, UiTextDocumentRevision},
};

// 同一节点编号可出现于不同树；会话绑定以树与节点共同识别编辑 owner。
/// 以树身份与节点身份共同索引会话状态。
/// 键拥有树标识；绑定、历史和同步错误均使用此键，不能仅凭节点编号跨树复用。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct UiTextDocumentBindingKey {
    pub(super) tree_id: UiTreeId,
    pub(super) node_id: UiNodeId,
}

// 文档 revision 与表面 source_epoch 是两套一致性标记；编辑前须同时核对，避免使用尚未同步的表面源。
/// 保存 owner 对应的文档身份、预期文档版本与已同步的 Surface 源 epoch。
/// 它是可复制的身份记录，不借用文本，也不保证未来版本仍有效；查询和编辑仍须核对 epoch 与存储版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiTextDocumentBinding {
    pub(super) document_id: UiTextDocumentId,
    pub(super) revision: UiTextDocumentRevision,
    pub(super) source_epoch: u64,
}
