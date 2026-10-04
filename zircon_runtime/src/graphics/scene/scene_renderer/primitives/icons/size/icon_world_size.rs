use crate::core::framework::render::OverlayBillboardIcon;

/// 贴图图标与缺图标线框回退共用世界尺寸，确保资源暂不可用时可视尺度不突变。
// TODO: [CR-W12-RENDER-AUX-A-0003] 贴图沿 right/up 各偏半尺寸，方向光回退十字端点偏全尺寸；
// 旧可视范围约定尚缺设计/截图证据，需在授权视觉验证中固定相机并切换素材可用性对照边界。
pub(in crate::graphics::scene::scene_renderer::primitives) fn icon_world_size(
    icon: &OverlayBillboardIcon,
) -> f32 {
    (icon.size * 0.0035).max(0.04)
}
