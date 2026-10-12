use serde::{Deserialize, Serialize};

/// 选择 Hybrid GI 的诊断视图；由渲染调试路径消费，不改变场景提取的权威内容。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderHybridGiDebugView {
    None,
    Cards,
    SurfaceCache,
    VoxelClipmap,
    InputSet,
}

impl Default for RenderHybridGiDebugView {
    fn default() -> Self {
        Self::None
    }
}
