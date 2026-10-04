use super::super::super::super::query as session_query;
use super::super::super::super::*;

impl RuntimeSessionArchive {
    /// 解析显式 ID 或时间/标签选择器并返回可持久化摘要；选择失败不改变档案。
    pub fn select_slot(
        &self,
        selector: RuntimeSessionSlotSelector,
    ) -> Result<RuntimeSessionSlotSelectionReport, RuntimeSessionArchiveError> {
        session_query::select_slot(self, selector)
    }
}
