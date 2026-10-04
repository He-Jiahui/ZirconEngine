use super::SceneViewportRenderPacket;

/// 对外保留提取包命名；其内容仍是轻量视口场景包，不等同完整 RenderFrameExtract。
pub type RenderExtractPacket = SceneViewportRenderPacket;
/// 预览、回读及合成验证路径使用的场景快照名称。
pub type RenderSceneSnapshot = SceneViewportRenderPacket;
