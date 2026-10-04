use std::sync::Arc;

use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
};
use crate::render_graph::QueueLane;

use super::IRRADIANCE_VOLUME_BIND_EXECUTOR_ID;

pub(super) fn registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![RenderPassExecutorRegistration::new_executor(
        IRRADIANCE_VOLUME_BIND_EXECUTOR_ID,
        Arc::new(IrradianceVolumeBindExecutor),
    )]
}

/// 图中的辐照度体积绑定边界。纹理选择与上传已在 compiled_scene_frame_foundation
/// 准备帧资源时完成，执行器仅校验通道和 GPU 上下文，避免同帧重复准备。
struct IrradianceVolumeBindExecutor;

impl RenderPassExecutor for IrradianceVolumeBindExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        if context.pass_name != IRRADIANCE_VOLUME_BIND_EXECUTOR_ID
            || context.executor_id.as_str() != IRRADIANCE_VOLUME_BIND_EXECUTOR_ID
            || context.declared_queue != QueueLane::Graphics
        {
            return Err("irradiance.volume_bind executor contract mismatch".to_string());
        }
        context.require_gpu()?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/executor.rs"]
mod tests;
