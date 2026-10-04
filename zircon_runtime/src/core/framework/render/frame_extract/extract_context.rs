use super::super::SceneViewportExtractRequest;
use super::RenderWorldSnapshotHandle;

/// 场景生产端的一次提取请求，同时携带世界身份与视口配置。
/// `World` 和 `LevelSystem` 用它将同一来源的帧数据交给渲染边界。
#[derive(Clone, Debug, PartialEq)]
pub struct RenderExtractContext {
    pub world: RenderWorldSnapshotHandle,
    pub request: SceneViewportExtractRequest,
}

impl RenderExtractContext {
    pub fn new(world: RenderWorldSnapshotHandle, request: SceneViewportExtractRequest) -> Self {
        Self { world, request }
    }
}
