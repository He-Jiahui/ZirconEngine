use zircon_render_server::{RenderQualityProfile, RenderServerError, RenderViewportHandle};

use super::wgpu_render_server::WgpuRenderServer;

pub(in crate::runtime::server) fn set_quality_profile(
    server: &WgpuRenderServer,
    viewport: RenderViewportHandle,
    profile: RenderQualityProfile,
) -> Result<(), RenderServerError> {
    let mut state = server.state.lock().unwrap();
    let record = state
        .viewports
        .get_mut(&viewport)
        .ok_or(RenderServerError::UnknownViewport {
            viewport: viewport.raw(),
        })?;
    record.quality_profile = Some(profile.clone());
    state.stats.last_quality_profile = Some(profile.name);
    Ok(())
}
