use std::sync::Arc;

use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
};
use crate::render_graph::QueueLane;

use super::LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID;

pub(super) fn registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![RenderPassExecutorRegistration::new_executor(
        LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID,
        Arc::new(LightCookieAtlasBuildExecutor),
    )]
}

struct LightCookieAtlasBuildExecutor;

impl RenderPassExecutor for LightCookieAtlasBuildExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        if context.pass_name != LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID
            || context.executor_id.as_str() != LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID
            || context.declared_queue != QueueLane::Graphics
        {
            return Err("cookie.atlas_build executor contract mismatch".to_string());
        }
        let gpu = context.require_gpu()?;
        let cookies = gpu
            .frame_extract()
            .lighting
            .advanced_lighting
            .cookies
            .clone();
        let streamer = gpu
            .streamer
            .ok_or_else(|| "cookie.atlas_build requires resource streamer context".to_string())?;
        let mesh_pipelines = gpu
            .mesh_pipelines
            .as_deref_mut()
            .ok_or_else(|| "cookie.atlas_build requires mesh pipeline context".to_string())?;
        mesh_pipelines
            .light_cookies
            .rebuild(gpu.device, gpu.encoder, streamer, &cookies);
        Ok(())
    }
}
