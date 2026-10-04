use serde::{Deserialize, Serialize};

/// 控制探针捕获频率；按需模式依赖更新状态在内容变化后重新标脏。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanarUpdateMode {
    EveryFrame,
    #[default]
    OnDemand,
}
