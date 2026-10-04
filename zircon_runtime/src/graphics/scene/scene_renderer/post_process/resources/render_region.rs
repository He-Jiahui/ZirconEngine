use crate::graphics::types::ViewportRenderRegion;

/// 中间效果目标使用以零为原点的局部视口，避免把物理输出原点带入局部纹理。
pub(super) fn apply_local_render_region_to_pass(
    pass: &mut wgpu::RenderPass<'_>,
    render_region: ViewportRenderRegion,
) -> bool {
    render_region.apply_local_to_render_pass(pass)
}

/// 终端输出使用目标中的物理视口和裁剪区域；空区域返回 false，调用者应跳过 draw。
pub(super) fn apply_physical_render_region_to_pass(
    pass: &mut wgpu::RenderPass<'_>,
    render_region: ViewportRenderRegion,
) -> bool {
    render_region.apply_physical_to_render_pass(pass)
}
