use crate::scene::LevelMetadata;

/// 关卡恢复成功后的结果副本；实体数来自新世界，元数据是本次写入值。
/// 此报告不提供世界与元数据并发发布的联合快照保证。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSessionLevelRestoreReport {
    pub slot_id: String,
    pub metadata: LevelMetadata,
    pub entity_count: usize,
}
