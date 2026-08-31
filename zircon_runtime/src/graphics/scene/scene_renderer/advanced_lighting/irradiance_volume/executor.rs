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
mod tests {
    #[test]
    fn irradiance_bind_executor_does_not_repeat_frame_preparation() {
        let production = include_str!("executor.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("irradiance executor test boundary");

        assert!(production.contains("context.require_gpu()?"));
        assert!(!production.contains("select_irradiance_volume_for_view"));
        assert!(!production.contains("irradiance_volume_texture"));
        assert!(!production.contains(".prepare("));
        assert!(!production.contains("gpu.queue"));
    }
}
