use super::super::super::slot_mutation;
use super::super::super::*;

impl RuntimeSessionArchive {
    /// 在预览中严格验证档案与目标是否存在；报告只包含摘要，不取出或修改槽位。
    pub fn preview_remove_slot(
        &self,
        slot_id: &str,
    ) -> Result<RuntimeSessionSlotMutationPreviewReport, RuntimeSessionArchiveError> {
        slot_mutation::preview_remove_slot(self, slot_id)
    }
}
