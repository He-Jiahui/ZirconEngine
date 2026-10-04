use super::super::super::construction;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn empty() -> Self {
        construction::empty()
    }

    /// 接收已有槽位，规范化元数据并验证档案；重复或非规范槽位 ID 会返回错误。
    pub fn from_slots(slots: Vec<RuntimeSessionSlot>) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_slots(slots)
    }
}

impl Default for RuntimeSessionArchive {
    fn default() -> Self {
        Self::empty()
    }
}
