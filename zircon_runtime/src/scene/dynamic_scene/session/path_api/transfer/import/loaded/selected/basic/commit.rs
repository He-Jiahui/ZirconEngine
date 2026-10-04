use std::path::Path;

use super::super::super::super::super::super::super::{
    path_transfer, RuntimeSessionArchive, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 导入来源档案中选择器解析的槽位；新 ID 修剪后须非空且未被目标档案占用。
    /// 继承来源槽位元数据，包括标签和更新时间。目标路径须已存在；成功保存后返回目录摘要，读改写期间不提供跨调用事务隔离。
    pub fn import_selected_slot_from_archive_at_path_atomically(
        path: impl AsRef<Path>,
        incoming: &RuntimeSessionArchive,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<RuntimeSessionArchiveManifest, RuntimeSessionArchiveError> {
        path_transfer::import_selected_slot_from_archive_at_path_atomically(
            path,
            incoming,
            selector,
            new_slot_id,
        )
    }
}
