use super::super::super::{EnvironmentExtract, RenderOverlayExtract};
use super::super::virtual_geometry::RenderVirtualGeometryDebugState;
use super::{PreviewEnvironmentExtract, RenderSceneGeometryExtract};

/// 场景提取到视口预览的值快照，供 UI、回读及 RenderFrameExtract 的适配入口。
/// 正式场景提交应直接构建完整帧载荷，不能依赖此包恢复未承载的 sideband。
#[derive(Clone, Debug, PartialEq)]
pub struct SceneViewportRenderPacket {
    pub scene: RenderSceneGeometryExtract,
    pub overlays: RenderOverlayExtract,
    pub environment: EnvironmentExtract,
    pub preview: PreviewEnvironmentExtract,
    pub virtual_geometry_debug: Option<RenderVirtualGeometryDebugState>,
}
