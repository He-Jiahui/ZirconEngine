use crate::core::editor_message::DocumentId;

use super::SaveReason;
use crate::core::extension::toolkit::ToolkitInstanceId;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Toolkit hook 成功后的文档保存回执；字节数与写入保证来自 SaveCtx。
pub struct DocumentSaveReport {
    document: DocumentId,
    instance: ToolkitInstanceId,
    reason: SaveReason,
    written_bytes: u64,
    source_write_guarantee: DocumentSaveGuarantee,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// 保证只覆盖同一源写入 Authority 的协作写方；外部写方冲突仍需审慎处理。
pub(in crate::core::extension::toolkit) struct DocumentSaveGuarantee {
    cooperating_source_writes_are_serialized: bool,
    external_conflict_detection_is_best_effort: bool,
}

impl DocumentSaveGuarantee {
    pub(super) const fn serialized_project_source() -> Self {
        Self {
            cooperating_source_writes_are_serialized: true,
            external_conflict_detection_is_best_effort: true,
        }
    }
}

impl DocumentSaveReport {
    pub(in crate::core::extension::toolkit) const fn new(
        document: DocumentId,
        instance: ToolkitInstanceId,
        reason: SaveReason,
        written_bytes: u64,
        source_write_guarantee: DocumentSaveGuarantee,
    ) -> Self {
        Self {
            document,
            instance,
            reason,
            written_bytes,
            source_write_guarantee,
        }
    }

    pub const fn document_id(&self) -> DocumentId {
        self.document
    }

    pub fn instance_id(&self) -> &ToolkitInstanceId {
        &self.instance
    }

    pub const fn reason(&self) -> SaveReason {
        self.reason
    }

    pub const fn written_bytes(&self) -> u64 {
        self.written_bytes
    }

    pub const fn cooperating_source_writes_are_serialized(&self) -> bool {
        self.source_write_guarantee
            .cooperating_source_writes_are_serialized
    }

    pub const fn external_conflict_detection_is_best_effort(&self) -> bool {
        self.source_write_guarantee
            .external_conflict_detection_is_best_effort
    }
}
