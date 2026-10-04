use std::path::PathBuf;

use super::super::RuntimeSessionMetadata;

#[derive(Clone, Debug, PartialEq, Eq)]
/// 导入与同档案复制的当前预检摘要；计数为来源场景容器长度，报告不保留提交计划。
pub struct RuntimeSessionSlotImportPreviewReport {
    pub source_slot_id: String,
    pub destination_slot_id: String,
    pub metadata: RuntimeSessionMetadata,
    pub entity_count: usize,
    pub resource_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 单槽导出的当前预检摘要；无路径预览没有目标路径，替换标记不锁定稍后的文件状态。
pub struct RuntimeSessionSlotExportPreviewReport {
    pub source_slot_id: String,
    pub target_path: Option<PathBuf>,
    pub will_replace_target: bool,
    pub metadata: RuntimeSessionMetadata,
    pub entity_count: usize,
    pub resource_count: usize,
}
