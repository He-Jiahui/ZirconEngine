use std::sync::Arc;

use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutionContext, RenderPassExecutorRegistration,
};
use crate::render_graph::QueueLane;

mod integrate;
mod light_scatter;
mod media_inject;

pub const VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID: &str = "volumetric.media_inject";
pub const VOLUMETRIC_LIGHT_SCATTER_EXECUTOR_ID: &str = "volumetric.light_scatter";
pub const VOLUMETRIC_INTEGRATE_EXECUTOR_ID: &str = "volumetric.integrate";

pub(crate) fn registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new_executor(
            VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID,
            Arc::new(media_inject::VolumetricMediaInjectExecutor::default()),
        ),
        RenderPassExecutorRegistration::new_executor(
            VOLUMETRIC_LIGHT_SCATTER_EXECUTOR_ID,
            Arc::new(light_scatter::VolumetricLightScatterExecutor::default()),
        ),
        RenderPassExecutorRegistration::new_executor(
            VOLUMETRIC_INTEGRATE_EXECUTOR_ID,
            Arc::new(integrate::VolumetricIntegrateExecutor::default()),
        ),
    ]
}

fn validate_compute_context(
    context: &RenderPassExecutionContext<'_>,
    expected_executor_id: &str,
) -> Result<(), String> {
    if context.executor_id.as_str() != expected_executor_id
        || context.pass_name != expected_executor_id
    {
        return Err(format!(
            "volumetric executor contract mismatch: expected pass/executor `{expected_executor_id}`, got pass `{}` executor `{}`",
            context.pass_name, context.executor_id
        ));
    }
    if context.declared_queue != QueueLane::AsyncCompute {
        return Err(format!(
            "volumetric executor `{expected_executor_id}` requires AsyncCompute declaration, got `{:?}`",
            context.declared_queue
        ));
    }
    if context.queue != QueueLane::AsyncCompute && context.queue != QueueLane::Graphics {
        return Err(format!(
            "volumetric executor `{expected_executor_id}` cannot execute on `{:?}`",
            context.queue
        ));
    }
    Ok(())
}
