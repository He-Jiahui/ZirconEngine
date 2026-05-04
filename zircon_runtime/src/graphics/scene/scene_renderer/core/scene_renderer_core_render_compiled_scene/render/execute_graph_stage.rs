use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::pipeline::{CompiledRenderPipeline, CompiledRenderPipelinePassStage};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources, RenderPassExecutionContext,
    RenderPassExecutorId, RenderPassExecutorRegistry, RenderPassGpuExecutionContext,
};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene)
struct RenderGraphStageExecution
<'a> {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) resources:
        &'a mut RenderGraphExecutionResources,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) record:
        &'a mut RenderGraphExecutionRecord,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) plugin_outputs:
        &'a mut RenderPluginRendererOutputs,
}

impl<'a> RenderGraphStageExecution<'a> {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn new(
        resources: &'a mut RenderGraphExecutionResources,
        record: &'a mut RenderGraphExecutionRecord,
        plugin_outputs: &'a mut RenderPluginRendererOutputs,
    ) -> Self {
        Self {
            resources,
            record,
            plugin_outputs,
        }
    }
}

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn import_frame_targets(
    resources: &mut RenderGraphExecutionResources,
    target: &OffscreenTarget,
) {
    resources.import_texture_view(
        "scene-color",
        target
            .scene_color
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    resources.import_texture_view(
        "scene-depth",
        target
            .depth
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
}

#[allow(clippy::too_many_arguments)]
pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn execute_graph_stage(
    pipeline: &CompiledRenderPipeline,
    registry: &RenderPassExecutorRegistry,
    stage: RenderPassStage,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ViewportRenderFrame,
    scene_bind_group: &wgpu::BindGroup,
    execution: &mut RenderGraphStageExecution<'_>,
) -> Result<(), GraphicsError> {
    for stage_entry in pipeline
        .pass_stages
        .iter()
        .filter(|entry| entry.stage == stage)
    {
        execute_graph_pass(
            pipeline,
            registry,
            stage_entry,
            device,
            queue,
            encoder,
            frame,
            scene_bind_group,
            execution,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn execute_graph_pass(
    pipeline: &CompiledRenderPipeline,
    registry: &RenderPassExecutorRegistry,
    stage_entry: &CompiledRenderPipelinePassStage,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ViewportRenderFrame,
    scene_bind_group: &wgpu::BindGroup,
    execution: &mut RenderGraphStageExecution<'_>,
) -> Result<(), GraphicsError> {
    let Some(pass) = pipeline
        .graph
        .passes()
        .iter()
        .find(|pass| pass.name == stage_entry.pass_name)
    else {
        return Err(GraphicsError::Asset(format!(
            "compiled render pipeline `{}` records stage `{:?}` for missing pass `{}`",
            pipeline.name, stage_entry.stage, stage_entry.pass_name
        )));
    };
    if pass.culled {
        return Ok(());
    }
    let executor_id = pass.executor_id.as_ref().ok_or_else(|| {
        GraphicsError::Asset(format!("render pass `{}` has no executor id", pass.name))
    })?;
    let executor_id = RenderPassExecutorId::new(executor_id.clone());
    let gpu = RenderPassGpuExecutionContext::new(
        device,
        queue,
        encoder,
        frame,
        scene_bind_group,
        &mut *execution.resources,
        &mut *execution.plugin_outputs,
    );
    let mut context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            executor_id.clone(),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            pass.resources.clone(),
        )
        .with_gpu(gpu);

    registry
        .execute(&mut context)
        .map_err(GraphicsError::Asset)?;
    execution
        .record
        .push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            Some(stage_entry.stage),
            pass.name.clone(),
            executor_id.as_str().to_string(),
            pass.queue,
            pass.declared_queue,
            pass.dependencies.clone(),
            pass.resources.clone(),
        );
    Ok(())
}
