use zircon_render_server::{RenderServerError, RenderStats};

use super::super::wgpu_render_server::WgpuRenderServer;

pub(in crate::runtime::server) fn query_stats(
    server: &WgpuRenderServer,
) -> Result<RenderStats, RenderServerError> {
    Ok(server.state.lock().unwrap().stats.clone())
}
