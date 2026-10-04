use serde::{Deserialize, Serialize};

use super::{TabInsertionSide, ViewInstanceId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 把新标签放在现有实例的前后；目标必须仍在被附加的标签容器内，由布局管理器处理失效目标。
pub struct TabInsertionAnchor {
    pub target_id: ViewInstanceId,
    pub side: TabInsertionSide,
}
