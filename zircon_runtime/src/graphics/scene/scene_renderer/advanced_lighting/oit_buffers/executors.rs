use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE, PostProcessGraphResourceNames,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassDeviceEpochCache, RenderPassExecutionContext, RenderPassExecutor,
    RenderPassExecutorRegistration,
};
use crate::render_graph::{QueueLane, RenderGraphResourceAccessKind};

use super::OitFragmentStorePipeline;
use super::resolve_pipeline::OitResolvePipeline;
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
    pipeline: Mutex<RenderPassDeviceEpochCache<wgpu::TextureFormat, OitFragmentStorePipeline>>,
}

impl RenderPassExecutor for OitFragmentStoreExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_graphics_context(context, OIT_FRAGMENT_STORE_EXECUTOR_ID)?;
        let gpu = context.require_gpu()?;
        let device_epoch = gpu.device_epoch().ok_or_else(|| {
            "OIT fragment store requires a materialized device epoch before pipeline recording"
                .to_string()
        })?;
        let layers = gpu
            .require_buffer_binding(
                PostProcessGraphResourceNames::OIT_LAYERS,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        let counts = gpu
            .require_buffer_binding(
                PostProcessGraphResourceNames::OIT_COUNTS,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        let storage_bindings = [layers.clone(), counts.clone()];
        ensure_storage_binding_capacity(gpu.device, &storage_bindings)?;
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
        let mut pipeline_cache = self
            .pipeline
            .lock()
            .map_err(|_| "OIT fragment-store pipeline cache lock poisoned".to_string())?;
        let pipeline = pipeline_cache.get_or_try_insert_with(device_epoch, depth_format, || {
            let oit_layout = gpu
                .mesh_pipelines
                .as_deref()
                .ok_or_else(|| "OIT fragment store requires mesh pipeline context".to_string())?
                .oit_fragment_store_layout();
            Ok(OitFragmentStorePipeline::new(
                gpu.device,
                gpu.scene_bind_group_layout(),
                oit_layout,
                depth_format,
            ))
        })?;
        gpu.encoder.clear_buffer(
            counts.buffer,
            counts.offset,
            counts.size.map(std::num::NonZeroU64::get),
        );
        gpu.record_oit_fragment_store_to_resources(
            pipeline,
            PostProcessGraphResourceNames::SCENE_DEPTH,
            &layers,
            &counts,
            settings,
        )
    }
}

#[derive(Default)]
struct OitResolveExecutor {
    pipeline: Mutex<RenderPassDeviceEpochCache<wgpu::TextureFormat, OitResolvePipeline>>,
}

impl RenderPassExecutor for OitResolveExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_graphics_context(context, OIT_RESOLVE_EXECUTOR_ID)?;
        let gpu = context.require_gpu()?;
        let device_epoch = gpu.device_epoch().ok_or_else(|| {
            "OIT resolve requires a materialized device epoch before pipeline recording".to_string()
        })?;
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
            .require_buffer_binding(
                PostProcessGraphResourceNames::OIT_LAYERS,
                RenderGraphResourceAccessKind::Read,
            )?
            .clone();
        let counts = gpu
            .require_buffer_binding(
                PostProcessGraphResourceNames::OIT_COUNTS,
                RenderGraphResourceAccessKind::Read,
            )?
            .clone();
        let storage_bindings = [layers.clone(), counts.clone()];
        ensure_storage_binding_capacity(gpu.device, &storage_bindings)?;
        let scene_color = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        let target_format = gpu.target_format();
        let mut pipeline_cache = self
            .pipeline
            .lock()
            .map_err(|_| "OIT resolve pipeline cache lock poisoned".to_string())?;
        let pipeline =
            pipeline_cache.get_or_try_insert_with(device_epoch, target_format, || {
                Ok(OitResolvePipeline::new(gpu.device, target_format))
            })?;
        pipeline.encode(
            gpu.device,
            gpu.encoder,
            &scene_color,
            layers,
            counts,
            gpu.render_region(),
            settings,
        );
        Ok(())
    }
}

fn ensure_storage_binding_capacity(
    device: &wgpu::Device,
    bindings: &[wgpu::BufferBinding<'_>],
) -> Result<(), String> {
    let available = device.limits().max_storage_buffers_per_shader_stage;
    if available < OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE {
        return Err(format!(
            "OIT graph reached execution with {available} storage buffers per shader stage; at least {OIT_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE} are required"
        ));
    }
    let max_binding_size = u64::from(device.limits().max_storage_buffer_binding_size);
    if let Some(oversized) = bindings
        .iter()
        .map(|binding| {
            binding.size.map_or_else(
                || binding.buffer.size().saturating_sub(binding.offset),
                std::num::NonZeroU64::get,
            )
        })
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
