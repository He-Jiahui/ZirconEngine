use super::super::super::super::super::super::{
    slot_import, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 预览来源档案中选择器解析的槽位；新 ID 修剪后须非空且未被目标档案占用。
    /// 继承来源槽位元数据，包括标签和更新时间。报告仅描述当前检查结果，不保留后续提交计划。
    pub fn preview_import_selected_slot_from_archive(
        &self,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionSlotImportPreviewReport, RuntimeSessionArchiveError> {
        slot_import::preview_import_selected_slot_from_archive(
            self,
            incoming,
            selector,
            new_slot_id,
        )
    }
}
