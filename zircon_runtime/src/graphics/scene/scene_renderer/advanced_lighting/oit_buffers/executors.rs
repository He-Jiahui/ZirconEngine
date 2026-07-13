use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    PostProcessGraphResourceNames, OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
};
use crate::render_graph::{QueueLane, RenderGraphResourceAccessKind};

use super::resolve_pipeline::OitResolvePipeline;
use super::OitFragmentStorePipeline;
use super::{OIT_FRAGMENT_STORE_EXECUTOR_ID, OIT_RESOLVE_EXECUTOR_ID};

pub(crate) fn registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new_executor(
            OIT_FRAGMENT_STORE_EXECUTOR_ID,
            Arc::new(OitFragmentStoreExecutor::default()),
        ),
        RenderPassExecutorRegistration::new_executor(
            OIT_RESOLVE_EXECUTOR_ID,
            Arc::new(OitResolveExecutor::default()),
        ),
    ]
}

#[derive(Default)]
struct OitFragmentStoreExecutor {
    pipeline: Mutex<Option<OitFragmentStorePipeline>>,
}

impl RenderPassExecutor for OitFragmentStoreExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_graphics_context(context, OIT_FRAGMENT_STORE_EXECUTOR_ID)?;
        let gpu = context.require_gpu()?;
        let layers = gpu
            .require_buffer(
                PostProcessGraphResourceNames::OIT_LAYERS,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        let counts = gpu
            .require_buffer(
                PostProcessGraphResourceNames::OIT_COUNTS,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        ensure_storage_binding_capacity(gpu.device, [&layers, &counts])?;
        gpu.encoder.clear_buffer(&layers, 0, None);
        gpu.encoder.clear_buffer(&counts, 0, None);
        let settings = gpu
            .frame_extract()
            .lighting
            .advanced_lighting
            .oit
            .ok_or_else(|| {
                "OIT graph was scheduled without camera OitSettings; use sorted transparency"
                    .to_string()
            })?;
        let depth_format = gpu.depth_format();
        let mut pipeline = self
            .pipeline
            .lock()
            .map_err(|_| "OIT fragment-store pipeline cache lock poisoned".to_string())?;
        if pipeline
            .as_ref()
            .is_none_or(|cached| cached.depth_format() != depth_format)
        {
            let oit_layout = gpu
                .mesh_pipelines
                .as_deref()
                .ok_or_else(|| "OIT fragment store requires mesh pipeline context".to_string())?
                .oit_fragment_store_layout();
            *pipeline = Some(OitFragmentStorePipeline::new(
                gpu.device,
                gpu.scene_bind_group_layout(),
                oit_layout,
                depth_format,
            ));
        }
        gpu.record_oit_fragment_store_to_resources(
            pipeline.as_ref().unwrap(),
            PostProcessGraphResourceNames::SCENE_DEPTH,
            &layers,
            &counts,
            settings,
        )
    }
}

#[derive(Default)]
struct OitResolveExecutor {
    pipeline: Mutex<Option<OitResolvePipeline>>,
}

impl RenderPassExecutor for OitResolveExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_graphics_context(context, OIT_RESOLVE_EXECUTOR_ID)?;
        let gpu = context.require_gpu()?;
        let settings = gpu
            .frame_extract()
            .lighting
            .advanced_lighting
            .oit
            .ok_or_else(|| {
                "OIT graph was scheduled without camera OitSettings; use sorted transparency"
                    .to_string()
            })?;
        let layers = gpu
            .require_buffer(
                PostProcessGraphResourceNames::OIT_LAYERS,
                RenderGraphResourceAccessKind::Read,
            )?
            .clone();
        let counts = gpu
            .require_buffer(
                PostProcessGraphResourceNames::OIT_COUNTS,
                RenderGraphResourceAccessKind::Read,
            )?
            .clone();
        ensure_storage_binding_capacity(gpu.device, [&layers, &counts])?;
        let scene_color = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        let target_format = gpu.target_format();
        let mut pipeline = self
            .pipeline
            .lock()
            .map_err(|_| "OIT resolve pipeline cache lock poisoned".to_string())?;
        if pipeline
            .as_ref()
            .is_none_or(|cached| cached.target_format() != target_format)
        {
            *pipeline = Some(OitResolvePipeline::new(gpu.device, target_format));
        }
        pipeline.as_ref().unwrap().encode(
            gpu.device,
            gpu.encoder,
            &scene_color,
            &layers,
            &counts,
            gpu.render_region(),
            settings,
        );
        Ok(())
    }
}

fn ensure_storage_binding_capacity<'a>(
    device: &wgpu::Device,
    buffers: impl IntoIterator<Item = &'a wgpu::Buffer>,
) -> Result<(), String> {
    let available = device.limits().max_storage_buffers_per_shader_stage;
    if available < OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE {
        return Err(format!(
            "OIT graph reached execution with {available} storage buffers per shader stage; at least {OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE} are required"
        ));
    }
    let max_binding_size = u64::from(device.limits().max_storage_buffer_binding_size);
    if let Some(oversized) = buffers
        .into_iter()
        .map(wgpu::Buffer::size)
        .find(|size| *size > max_binding_size)
    {
        return Err(format!(
            "OIT buffer requires {oversized} bytes, exceeding max_storage_buffer_binding_size {max_binding_size}; use sorted transparency or reduce fragments_per_pixel_average"
        ));
    }
    Ok(())
}

fn validate_graphics_context(
    context: &RenderPassExecutionContext<'_>,
    expected_executor_id: &str,
) -> Result<(), String> {
    if context.executor_id.as_str() != expected_executor_id
        || context.pass_name != expected_executor_id
    {
        return Err(format!(
            "OIT executor contract mismatch: expected pass/executor `{expected_executor_id}`, got pass `{}` executor `{}`",
            context.pass_name, context.executor_id
        ));
    }
    if context.declared_queue != QueueLane::Graphics || context.queue != QueueLane::Graphics {
        return Err(format!(
            "OIT executor `{expected_executor_id}` requires the graphics queue"
        ));
    }
    debug_assert_eq!(OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE, 3);
    Ok(())
}
