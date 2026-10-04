use super::{RenderExtractContext, RenderFrameExtract};

/// 场景到渲染帧的生产契约；调用方应先确定世界句柄和视口请求。
/// `World` 负责场景快照，`LevelSystem` 还合并同代的动画姿态。
pub trait RenderExtractProducer {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract;
}
