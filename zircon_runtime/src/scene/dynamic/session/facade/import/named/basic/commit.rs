use super::super::super::super::super::super::{
    slot_import, RuntimeSessionArchive, RuntimeSessionArchiveError,
};

impl RuntimeSessionArchive {
    /// 导入来源档案中指定 ID 的槽位；新 ID 修剪后须非空且未被目标档案占用。
    /// 继承来源槽位元数据，包括标签和更新时间。复制场景并追加目标槽位，来源保持原样。
    pub fn import_slot_from_archive(
        &mut self,
        incoming: &RuntimeSessionArchive,
        source_slot_id: &str,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_import::import_slot_from_archive(self, incoming, source_slot_id, new_slot_id)
    }
}
