use std::sync::{Arc, Mutex};

use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassDeviceEpoch, RenderPassDeviceEpochCache, RenderPassExecutionContext,
    RenderPassExecutor, RenderPassExecutorRegistration, RenderPassGpuRecordingContext,
};
use crate::render_graph::{QueueLane, RenderGraphResourceAccessKind};

use super::pipelines::SubsurfacePipelines;
use super::prepared_frame::PreparedSubsurfaceFrame;
use super::{
    SSS_RECOMBINE_EXECUTOR_ID, SSS_SCATTER_EXECUTOR_ID, SSS_SETUP_EXECUTOR_ID,
    SSS_SETUP_PIPELINE_LABEL, SSS_TILE_SIZE,
};

#[derive(Default)]
struct SubsurfaceExecutor {
    pipelines: Mutex<RenderPassDeviceEpochCache<wgpu::TextureFormat, SubsurfacePipelines>>,
}

impl SubsurfaceExecutor {
    fn with_pipelines<T, C: RenderPassGpuRecordingContext>(
        &self,
        context: &mut C,
        device_epoch: RenderPassDeviceEpoch,
        target_format: wgpu::TextureFormat,
        use_pipeline: impl FnOnce(&SubsurfacePipelines, &mut C) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut pipeline_cache = self
            .pipelines
            .lock()
            .map_err(|_| "subsurface pipeline cache lock poisoned".to_string())?;
        let pipelines =
            pipeline_cache.get_or_try_insert_with(device_epoch, target_format, || {
                Ok(SubsurfacePipelines::new(
                    context.resource_factory(),
                    target_format,
                ))
            })?;
        use_pipeline(pipelines, context)
    }
}

struct SetupExecutor(Arc<SubsurfaceExecutor>);
struct ScatterExecutor(Arc<SubsurfaceExecutor>);
struct RecombineExecutor(Arc<SubsurfaceExecutor>);

impl RenderPassExecutor for SetupExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_context(context, SSS_SETUP_EXECUTOR_ID, QueueLane::AsyncCompute)?;
        let pass_name = context.pass_name.clone();
        let executor_id = context.executor_id.as_str().to_string();
        let gpu = context.require_gpu()?;
        let device_epoch = gpu.device_epoch().ok_or_else(|| {
            "subsurface setup requires a materialized device epoch before pipeline recording"
                .to_string()
        })?;
        let material = texture(gpu, PostProcessGraphResourceNames::GBUFFER_MATERIAL, true)?;
        let normal = texture(gpu, PostProcessGraphResourceNames::GBUFFER_NORMAL, true)?;
        let tile_list = buffer(gpu, PostProcessGraphResourceNames::SSS_TILE_LIST, false)?;
        let indirect_args = buffer(gpu, PostProcessGraphResourceNames::SSS_INDIRECT_ARGS, false)?;
        let params = buffer(gpu, PostProcessGraphResourceNames::SSS_PARAMS, false)?;
        let profiles = buffer(gpu, PostProcessGraphResourceNames::SSS_PROFILES, false)?;
        let params_buffer = params.buffer.clone();
        let profiles_buffer = profiles.buffer.clone();
        let size = gpu.viewport_size();
        let prepared = PreparedSubsurfaceFrame::prepare(gpu.frame_extract(), size)?;
        let dispatch = prepared.dispatch();
        let target_format = gpu.target_format();
        {
            let mut native = gpu.native_context();
            self.0.with_pipelines(
                &mut native,
                device_epoch,
                target_format,
                |pipelines, native| {
                    pipelines.encode_setup(
                        native,
                        &material,
                        &normal,
                        tile_list,
                        indirect_args,
                        params,
                        dispatch,
                    );
                    Ok(())
                },
            )?;
        }
        let mut uploads = prepared.buffer_uploads(params_buffer, profiles_buffer)?;
        gpu.append_pre_submit_buffer_uploads(&mut uploads);
        gpu.record_compute_dispatch_with_uploaded_bytes(
            pass_name,
            executor_id,
            SSS_SETUP_PIPELINE_LABEL,
            SSS_TILE_SIZE,
            dispatch,
            PreparedSubsurfaceFrame::uploaded_byte_len(),
            vec![
                PostProcessGraphResourceNames::SSS_TILE_LIST.to_string(),
                PostProcessGraphResourceNames::SSS_INDIRECT_ARGS.to_string(),
            ],
        );
        Ok(())
    }
}

impl RenderPassExecutor for ScatterExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_context(context, SSS_SCATTER_EXECUTOR_ID, QueueLane::AsyncCompute)?;
        let pass_name = context.pass_name.clone();
        let executor_id = context.executor_id.as_str().to_string();
        let gpu = context.require_gpu()?;
        let device_epoch = gpu.device_epoch().ok_or_else(|| {
            "subsurface scatter requires a materialized device epoch before pipeline recording"
                .to_string()
        })?;
        let diffuse = texture(gpu, PostProcessGraphResourceNames::SSS_DIFFUSE, true)?;
        let depth = texture(gpu, PostProcessGraphResourceNames::SCENE_DEPTH, true)?;
        let material = texture(gpu, PostProcessGraphResourceNames::GBUFFER_MATERIAL, true)?;
        let normal = texture(gpu, PostProcessGraphResourceNames::GBUFFER_NORMAL, true)?;
        let scattered = texture(gpu, PostProcessGraphResourceNames::SSS_SCATTERED, false)?;
        let tile_list = buffer(gpu, PostProcessGraphResourceNames::SSS_TILE_LIST, true)?;
        let indirect_args = buffer(gpu, PostProcessGraphResourceNames::SSS_INDIRECT_ARGS, true)?;
        let params = buffer(gpu, PostProcessGraphResourceNames::SSS_PARAMS, true)?;
        let profiles = buffer(gpu, PostProcessGraphResourceNames::SSS_PROFILES, true)?;
        let target_format = gpu.target_format();
        {
            let mut native = gpu.native_context();
            self.0.with_pipelines(
                &mut native,
                device_epoch,
                target_format,
                |pipelines, native| {
                    pipelines.encode_scatter(
                        native,
                        &diffuse,
                        &depth,
                        &material,
                        &normal,
                        tile_list,
                        indirect_args,
                        profiles,
                        params,
                        &scattered,
                    );
                    Ok(())
                },
            )?;
        }
        gpu.record_indirect_compute_dispatch(
            pass_name,
            executor_id,
            super::SSS_SCATTER_PIPELINE_LABEL,
            SSS_TILE_SIZE,
            vec![PostProcessGraphResourceNames::SSS_SCATTERED.to_string()],
        );
        Ok(())
    }
}

impl RenderPassExecutor for RecombineExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_context(context, SSS_RECOMBINE_EXECUTOR_ID, QueueLane::Graphics)?;
        let gpu = context.require_gpu()?;
        let device_epoch = gpu.device_epoch().ok_or_else(|| {
            "subsurface recombine requires a materialized device epoch before pipeline recording"
                .to_string()
        })?;
        let scattered = texture(gpu, PostProcessGraphResourceNames::SSS_SCATTERED, true)?;
        let specular = texture(gpu, PostProcessGraphResourceNames::SSS_SPECULAR, true)?;
        let material = texture(gpu, PostProcessGraphResourceNames::GBUFFER_MATERIAL, true)?;
        let scene_color = texture(gpu, PostProcessGraphResourceNames::SCENE_COLOR, false)?;
        let render_region =
            gpu.render_region_for_write_resource(PostProcessGraphResourceNames::SCENE_COLOR);
        let target_format = gpu.target_format();
        let mut native = gpu.native_context();
        self.0.with_pipelines(
            &mut native,
            device_epoch,
            target_format,
            |pipelines, native| {
                pipelines.encode_recombine(
                    native,
                    render_region,
                    &scattered,
                    &specular,
                    &material,
                    &scene_color,
                );
                Ok(())
            },
        )
    }
}

pub(super) fn registrations() -> Vec<RenderPassExecutorRegistration> {
    let shared = Arc::new(SubsurfaceExecutor::default());
    vec![
        RenderPassExecutorRegistration::new_executor(
            SSS_SETUP_EXECUTOR_ID,
            Arc::new(SetupExecutor(Arc::clone(&shared))),
        ),
        RenderPassExecutorRegistration::new_executor(
            SSS_SCATTER_EXECUTOR_ID,
            Arc::new(ScatterExecutor(Arc::clone(&shared))),
        ),
        RenderPassExecutorRegistration::new_executor(
            SSS_RECOMBINE_EXECUTOR_ID,
            Arc::new(RecombineExecutor(shared)),
        ),
    ]
}

fn validate_context(
    context: &RenderPassExecutionContext<'_>,
    expected: &str,
    queue: QueueLane,
) -> Result<(), String> {
    if context.pass_name != expected || context.executor_id.as_str() != expected {
        return Err(format!(
            "subsurface executor contract mismatch: expected `{expected}`, got pass `{}` executor `{}`",
            context.pass_name, context.executor_id
        ));
    }
    if context.declared_queue != queue {
        return Err(format!(
            "subsurface executor `{expected}` requires `{queue:?}`, got `{:?}`",
            context.declared_queue
        ));
    }
    Ok(())
}

fn texture(
    gpu: &crate::graphics::scene::scene_renderer::graph_execution::RenderPassGpuExecutionContext<
        '_,
    >,
    name: &str,
    read: bool,
) -> Result<wgpu::TextureView, String> {
    gpu.require_texture_view(
        name,
        if read {
            RenderGraphResourceAccessKind::Read
        } else {
            RenderGraphResourceAccessKind::Write
        },
    )
    .cloned()
}

fn buffer<'a>(
    gpu: &crate::graphics::scene::scene_renderer::graph_execution::RenderPassGpuExecutionContext<
        'a,
    >,
    name: &str,
    read: bool,
) -> Result<wgpu::BufferBinding<'a>, String> {
    gpu.require_buffer_binding(
        name,
        if read {
            RenderGraphResourceAccessKind::Read
        } else {
            RenderGraphResourceAccessKind::Write
        },
    )
}
