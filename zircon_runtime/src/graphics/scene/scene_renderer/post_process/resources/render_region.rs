use crate::graphics::types::ViewportRenderRegion;

pub(super) fn apply_local_render_region_to_pass(
    pass: &mut wgpu::RenderPass<'_>,
    render_region: ViewportRenderRegion,
) -> bool {
    render_region.apply_local_to_render_pass(pass)
}

pub(super) fn apply_physical_render_region_to_pass(
    pass: &mut wgpu::RenderPass<'_>,
    render_region: ViewportRenderRegion,
) -> bool {
    render_region.apply_physical_to_render_pass(pass)
}
