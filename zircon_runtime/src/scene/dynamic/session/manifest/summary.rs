use serde::{Deserialize, Serialize};

use super::super::RuntimeSessionMetadata;

/// 槽位的轻量目录项；供选择器、保存预览和 UI 查询使用，不携带场景实体数据。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSessionSlotSummary {
    pub slot_id: String,
    #[serde(default)]
    pub metadata: RuntimeSessionMetadata,
    pub scene_format_version: u32,
    pub entity_count: usize,
    pub resource_count: usize,
}
