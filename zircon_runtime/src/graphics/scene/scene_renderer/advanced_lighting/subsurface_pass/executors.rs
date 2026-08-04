use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    PostProcessGraphResourceNames, ViewProjectionMatrixPair, resolve_subsurface_profile_table,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
};
use crate::render_graph::{QueueLane, RenderGraphResourceAccessKind};

use super::pipelines::SubsurfacePipelines;
use super::{
    SSS_RECOMBINE_EXECUTOR_ID, SSS_SCATTER_EXECUTOR_ID, SSS_SETUP_EXECUTOR_ID,
    SSS_SETUP_PIPELINE_LABEL, SSS_TILE_SIZE,
};

#[derive(Default)]
struct SubsurfaceExecutor {
    pipelines: Mutex<Option<SubsurfacePipelines>>,
}

impl SubsurfaceExecutor {
    fn with_pipelines<T>(
        &self,
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        use_pipeline: impl FnOnce(&SubsurfacePipelines) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut pipelines = self
            .pipelines
            .lock()
            .map_err(|_| "subsurface pipeline cache lock poisoned".to_string())?;
        if pipelines
            .as_ref()
            .is_none_or(|pipelines| pipelines.target_format() != target_format)
        {
            *pipelines = Some(SubsurfacePipelines::new(device, target_format));
        }
        use_pipeline(pipelines.as_ref().unwrap())
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
        let material = texture(gpu, PostProcessGraphResourceNames::GBUFFER_MATERIAL, true)?;
        let normal = texture(gpu, PostProcessGraphResourceNames::GBUFFER_NORMAL, true)?;
        let tile_list = buffer(gpu, PostProcessGraphResourceNames::SSS_TILE_LIST, false)?;
        let indirect_args = buffer(gpu, PostProcessGraphResourceNames::SSS_INDIRECT_ARGS, false)?;
        let size = gpu.viewport_size();
        let profiles = &gpu
            .frame_extract()
            .lighting
            .advanced_lighting
            .subsurface_profiles;
        let table = resolve_subsurface_profile_table(profiles);
        let inverse_view_projection = inverse_view_projection(gpu, size);
        let dispatch = [size.x.max(1).div_ceil(8), size.y.max(1).div_ceil(8), 1];
        self.0
            .with_pipelines(gpu.device, gpu.target_format(), |pipelines| {
                pipelines.encode_setup(
                    gpu.device,
                    gpu.queue,
                    gpu.encoder,
                    size,
                    &material,
                    &normal,
                    &tile_list,
                    &indirect_args,
                    table.profiles.len() as u32,
                    table.active_profile_mask,
                    inverse_view_projection,
                    dispatch,
                );
                Ok(())
            })?;
        gpu.record_compute_dispatch_with_uploaded_bytes(
            pass_name,
            executor_id,
            SSS_SETUP_PIPELINE_LABEL,
            SSS_TILE_SIZE,
            dispatch,
            16,
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
        let diffuse = texture(gpu, PostProcessGraphResourceNames::SSS_DIFFUSE, true)?;
        let depth = texture(gpu, PostProcessGraphResourceNames::SCENE_DEPTH, true)?;
        let material = texture(gpu, PostProcessGraphResourceNames::GBUFFER_MATERIAL, true)?;
        let normal = texture(gpu, PostProcessGraphResourceNames::GBUFFER_NORMAL, true)?;
        let scattered = texture(gpu, PostProcessGraphResourceNames::SSS_SCATTERED, false)?;
        let tile_list = buffer(gpu, PostProcessGraphResourceNames::SSS_TILE_LIST, true)?;
        let indirect_args = buffer(gpu, PostProcessGraphResourceNames::SSS_INDIRECT_ARGS, true)?;
        let profiles = gpu
            .frame_extract()
            .lighting
            .advanced_lighting
            .subsurface_profiles
            .clone();
        let size = gpu.viewport_size();
        let inverse_view_projection = inverse_view_projection(gpu, size);
        self.0
            .with_pipelines(gpu.device, gpu.target_format(), |pipelines| {
                pipelines.encode_scatter(
                    gpu.device,
                    gpu.encoder,
                    size,
                    &profiles,
                    inverse_view_projection,
                    &diffuse,
                    &depth,
                    &material,
                    &normal,
                    &tile_list,
                    &indirect_args,
                    &scattered,
                )
            })?;
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

fn inverse_view_projection(
    gpu: &crate::graphics::scene::scene_renderer::graph_execution::RenderPassGpuExecutionContext<
        '_,
    >,
    size: crate::core::math::UVec2,
) -> crate::core::math::Mat4 {
    let camera = gpu.frame_extract().view.selected_effective_camera();
    ViewProjectionMatrixPair::from_camera(&camera, size)
        .clip_from_world_jittered
        .inverse()
}

impl RenderPassExecutor for RecombineExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_context(context, SSS_RECOMBINE_EXECUTOR_ID, QueueLane::Graphics)?;
        let gpu = context.require_gpu()?;
        let scattered = texture(gpu, PostProcessGraphResourceNames::SSS_SCATTERED, true)?;
        let specular = texture(gpu, PostProcessGraphResourceNames::SSS_SPECULAR, true)?;
        let material = texture(gpu, PostProcessGraphResourceNames::GBUFFER_MATERIAL, true)?;
        let scene_color = texture(gpu, PostProcessGraphResourceNames::SCENE_COLOR, false)?;
        let render_region =
            gpu.render_region_for_write_resource(PostProcessGraphResourceNames::SCENE_COLOR);
        self.0
            .with_pipelines(gpu.device, gpu.target_format(), |pipelines| {
                pipelines.encode_recombine(
                    gpu.device,
                    gpu.encoder,
                    render_region,
                    &scattered,
                    &specular,
                    &material,
                    &scene_color,
                );
                Ok(())
            })
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

fn buffer(
    gpu: &crate::graphics::scene::scene_renderer::graph_execution::RenderPassGpuExecutionContext<
        '_,
    >,
    name: &str,
    read: bool,
) -> Result<wgpu::Buffer, String> {
    gpu.require_buffer(
        name,
        if read {
            RenderGraphResourceAccessKind::Read
        } else {
            RenderGraphResourceAccessKind::Write
        },
    )
    .cloned()
}
