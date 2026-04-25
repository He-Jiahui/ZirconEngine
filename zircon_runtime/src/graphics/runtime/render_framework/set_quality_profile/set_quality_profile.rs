use crate::core::framework::render::{
    RenderFrameworkError, RenderQualityProfile, RenderViewportHandle,
};

use super::super::capability_validation::validate_quality_profile_capabilities;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn set_quality_profile(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    profile: RenderQualityProfile,
) -> Result<(), RenderFrameworkError> {
    let mut state = server.state.lock().unwrap();
    let capabilities = state.stats.capabilities.clone();
    let record =
        state
            .viewports
            .get_mut(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?;
    validate_quality_profile_capabilities(record.pipeline, &profile, &capabilities)?;
    record.quality_profile = Some(profile.clone());
    state.stats.last_quality_profile = Some(profile.name);
    Ok(())
}
