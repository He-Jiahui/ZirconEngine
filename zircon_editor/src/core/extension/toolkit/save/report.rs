use crate::core::editor_message::DocumentId;

use super::SaveReason;
use crate::core::extension::toolkit::ToolkitInstanceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSaveReport {
    document: DocumentId,
    instance: ToolkitInstanceId,
    reason: SaveReason,
    written_bytes: u64,
}

impl DocumentSaveReport {
    pub(crate) const fn new(
        document: DocumentId,
        instance: ToolkitInstanceId,
        reason: SaveReason,
        written_bytes: u64,
    ) -> Self {
        Self {
            document,
            instance,
            reason,
            written_bytes,
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
}
