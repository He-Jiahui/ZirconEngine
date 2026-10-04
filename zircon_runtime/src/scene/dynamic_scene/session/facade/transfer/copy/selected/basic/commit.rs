use super::super::super::super::super::super::{
    slot_copy, RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 在当前档案解析选择器后复制到新 ID，继承全部源元数据且不刷新时间；保留源槽位。
    /// 新 ID 修剪后须非空且未占用。
    pub fn copy_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        new_slot_id: impl Into<String>,
    ) -> Result<(), RuntimeSessionArchiveError> {
        slot_copy::copy_selected_slot(self, selector, new_slot_id)
    }
}
