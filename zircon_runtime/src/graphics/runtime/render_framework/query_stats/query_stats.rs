use crate::core::framework::render::{RenderFrameworkError, RenderStats};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn query_stats(
    server: &WgpuRenderFramework,
) -> Result<RenderStats, RenderFrameworkError> {
    Ok(server.state.lock().unwrap().stats.clone())
}
