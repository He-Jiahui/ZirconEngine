//! 一个会话槽位绑定场景快照与持久化元数据；具体捕获、比较、恢复委托给子模块。

mod capture;
mod diff;
mod restore;
mod scene_document;
mod summary;

use serde::{Deserialize, Serialize};

use super::super::DynamicScene;
use super::RuntimeSessionMetadata;

/// 命名的场景检查点；scene 使用版本化文档格式，档案在接纳槽位时验证 ID 与场景契约。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSessionSlot {
    pub slot_id: String,
    #[serde(default)]
    pub metadata: RuntimeSessionMetadata,
    #[serde(with = "scene_document")]
    pub scene: DynamicScene,
}
