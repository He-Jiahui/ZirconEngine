use std::sync::Arc;

use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
};
use crate::render_graph::QueueLane;

use super::LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID;

const MISSING_STREAMER_CONTEXT: &str = "cookie.atlas_build requires resource streamer context";
const MISSING_MESH_PIPELINE_CONTEXT: &str = "cookie.atlas_build requires mesh pipeline context";

pub(super) fn registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![RenderPassExecutorRegistration::new_executor(
        LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID,
        Arc::new(LightCookieAtlasBuildExecutor),
    )]
}

/// 在图执行期间把帧提取的 cookie 写入渲染器图集。先检查流送器和管线资源，
/// 再克隆本帧 cookie，避免错误上下文仍支付大规模提取成本。
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
        validate_light_cookie_executor_context(
            gpu.streamer.is_some(),
            gpu.mesh_pipelines.is_some(),
        )
        .map_err(str::to_string)?;
        let cookies = gpu
            .frame_extract()
            .lighting
            .advanced_lighting
            .cookies
            .clone();
        let streamer = gpu
            .streamer
            .expect("light cookie context preflight checked the resource streamer");
        let mesh_pipelines = gpu
            .mesh_pipelines
            .as_deref_mut()
            .expect("light cookie context preflight checked the mesh pipelines");
        mesh_pipelines
            .light_cookies
            .rebuild(gpu.device, gpu.encoder, streamer, &cookies);
        Ok(())
    }
}

fn validate_light_cookie_executor_context(
    has_streamer: bool,
    has_mesh_pipelines: bool,
) -> Result<(), &'static str> {
    if !has_streamer {
        return Err(MISSING_STREAMER_CONTEXT);
    }
    if !has_mesh_pipelines {
        return Err(MISSING_MESH_PIPELINE_CONTEXT);
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/executor_optimization_batch_gy_runtime580_tests.rs"]
mod optimization_batch_gy_runtime580_tests;
