use serde::{Deserialize, Serialize};

use super::{EntityId, WorldHandle};

/// 独立拥有的关卡概览快照，读出后不会随 World 后续变化自动更新。
/// 直接构造或 serde 回读不会核实句柄及实体是否仍存在，参见 `LevelManager::level_summary`。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelSummary {
    /// 生成概览时对应的关卡键。
    pub handle: WorldHandle,
    /// 生成概览时的实体数量。
    pub entity_count: usize,
    /// 可选活动相机实体键，不执行实体存在或相机组件校验。
    pub active_camera: Option<EntityId>,
}
