use super::super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlotSelector,
};

impl RuntimeSessionArchive {
    /// 设置当前选中槽位的更新时间，保留其场景和其他元数据；时间由调用者提供。
    /// 更新时间会影响后续按最新或最旧选择；本接口不校验时间是否单调，也不自动读取时钟。
    pub fn touch_selected_slot(
        &mut self,
        selector: RuntimeSessionSlotSelector,
        updated_at_unix_millis: u64,
    ) -> Result<(), RuntimeSessionArchiveError> {
        let report = self.select_slot(selector)?;
        self.touch_slot(&report.selected_slot_id, updated_at_unix_millis)
    }
}
