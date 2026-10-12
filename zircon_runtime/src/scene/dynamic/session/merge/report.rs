/// 按来源槽位 ID 的规范顺序分别记录插入、替换和跳过项；预览与提交共用此报告类型。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchiveMergeReport {
    pub inserted_slot_ids: Vec<String>,
    pub replaced_slot_ids: Vec<String>,
    pub skipped_slot_ids: Vec<String>,
}

impl RuntimeSessionArchiveMergeReport {
    /// 三类记录均为空才返回 true；仅有跳过项的报告仍非空，即使目标没有任何修改。
    pub fn is_empty(&self) -> bool {
        self.inserted_slot_ids.is_empty()
            && self.replaced_slot_ids.is_empty()
            && self.skipped_slot_ids.is_empty()
    }
}
