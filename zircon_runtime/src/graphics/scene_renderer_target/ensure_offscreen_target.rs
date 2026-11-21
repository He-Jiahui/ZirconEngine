use crate::core::framework::render::RenderViewportHandle;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::types::ViewportRenderOutputTarget;

/// 配置离屏目标；仅场景附件重建时返回 true，通知调用方失效尺寸相关缓存。
pub(crate) fn ensure_offscreen_target(
    device: &wgpu::Device,
    target: &mut Option<OffscreenTarget>,
    size: crate::core::math::UVec2,
    render_size: crate::core::math::UVec2,
    viewport: Option<RenderViewportHandle>,
    offscreen_output: Option<ViewportRenderOutputTarget>,
) -> bool {
    OffscreenTarget::ensure_allocations(
        device,
        target,
        size,
        render_size,
        viewport,
        offscreen_output,
    )
}

#[cfg(test)]
#[path = "tests/offscreen_final_retention.rs"]
mod tests;
