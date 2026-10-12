use serde::{Deserialize, Serialize};

/// 会话槽位选择意图；显式 ID 或按更新时间与标签选择，解析结果随后绑定到档案代际。
/// 构造器会修剪输入；直接反序列化的 SlotId 按原字符串精确匹配，标签查询在解析时修剪。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeSessionSlotSelector {
    SlotId { slot_id: String },
    LatestUpdated,
    OldestUpdated,
    LatestUpdatedWithTag { tag: String },
    OldestUpdatedWithTag { tag: String },
}
