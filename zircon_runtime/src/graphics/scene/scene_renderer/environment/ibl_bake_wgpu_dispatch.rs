use crate::core::framework::render::IblBakeArtifactRequest;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphComputeDispatchRecord, RenderPassExecutionContext, RenderPassGpuExecutionContext,
};
use crate::render_graph::RenderGraphResourceAccessKind;

use super::ibl_bake_graph_plan::{
    ibl_bake_pmrem_mip_from_pass_name, IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID,
    IBL_BAKE_IRRADIANCE_CUBE_PASS, IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID,
    IBL_BAKE_IRRADIANCE_SH9_PASS, IBL_BAKE_PMREM_EXECUTOR_ID, IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
};
use super::ibl_bake_shader_plan::IblBakeComputeKernelKind;
use super::ibl_bake_wgpu_binding::{
    create_ibl_bake_wgpu_bind_group, create_ibl_bake_wgpu_params_buffer,
    create_ibl_bake_wgpu_source_sampler, IblBakeWgpuOutputBindingResource,
};
use super::ibl_bake_wgpu_command_plan::{
    ibl_bake_wgpu_command_plan_for_request, IblBakeWgpuCommandPlan, IblBakeWgpuOutputPlan,
};
use super::ibl_bake_wgpu_pipeline_cache::create_ibl_bake_wgpu_compute_pipeline_from_cached_parts;

const IBL_BAKE_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct IblBakeWgpuEncodedDispatch {
    pub pipeline_label: String,
    pub dispatch_groups: [u32; 3],
}

pub(in crate::graphics::scene::scene_renderer) fn record_ibl_bake_wgpu_pass_for_request(
    context: &mut RenderPassExecutionContext<'_>,
    request: &IblBakeArtifactRequest,
) -> Result<IblBakeWgpuEncodedDispatch, String> {
    let command_plan = ibl_bake_wgpu_command_plan_for_request(request);
    let command = command_plan
        .commands
        .iter()
        .find(|command| command_matches_context(command, context))
        .ok_or_else(|| {
            format!(
                "no IBL bake WGPU command matches pass `{}` executor `{}`",
                context.pass_name, context.executor_id
            )
        })?;

    record_ibl_bake_wgpu_command(context, command)
}

pub(in crate::graphics::scene::scene_renderer) fn create_ibl_bake_wgpu_compute_pipeline(
    device: &wgpu::Device,
    command: &IblBakeWgpuCommandPlan,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader_label = format!("{}-shader", command.pipeline_label);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(shader_label.as_str()),
        source: wgpu::ShaderSource::Wgsl(command.wgsl_source.into()),
    });
    let layout_label = format!("{}-pipeline-layout", command.pipeline_label);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(layout_label.as_str()),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    create_ibl_bake_wgpu_compute_pipeline_from_cached_parts(
        device,
        command,
        &pipeline_layout,
        &shader,
    )
}

pub(in crate::graphics::scene::scene_renderer) fn encode_ibl_bake_wgpu_compute_dispatch(
    encoder: &mut wgpu::CommandEncoder,
    command: &IblBakeWgpuCommandPlan,
    pipeline: &wgpu::ComputePipeline,
    bind_group: &wgpu::BindGroup,
) -> Result<IblBakeWgpuEncodedDispatch, String> {
    validate_dispatch_groups(command)?;
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some(command.pipeline_label.as_str()),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.dispatch_workgroups(
        command.dispatch_groups[0],
        command.dispatch_groups[1],
        command.dispatch_groups[2],
    );
    drop(pass);

    Ok(IblBakeWgpuEncodedDispatch {
        pipeline_label: command.pipeline_label.clone(),
        dispatch_groups: command.dispatch_groups,
    })
}

fn record_ibl_bake_wgpu_command(
    context: &mut RenderPassExecutionContext<'_>,
    command: &IblBakeWgpuCommandPlan,
) -> Result<IblBakeWgpuEncodedDispatch, String> {
    let output_resource_name = output_resource_name(command);
    require_context_access(
        context,
        IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
        RenderGraphResourceAccessKind::Read,
    )?;
    require_context_access(
        context,
        output_resource_name,
        RenderGraphResourceAccessKind::Write,
    )?;

    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let resource_accesses = context.resources.clone();
    let encoded = {
        let gpu = context.require_gpu()?;
        let source_cubemap_view = RenderPassGpuExecutionContext::require_texture_view_by_name(
            &*gpu.resources,
            gpu.resource_resolver(),
            IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
            RenderGraphResourceAccessKind::Read,
        )?
        .clone();
        let output = resolve_output_binding(gpu, command)?;
        let params = create_ibl_bake_wgpu_params_buffer(gpu.device, command);
        let source_sampler = create_ibl_bake_wgpu_source_sampler(gpu.device);
        let (bind_group, pipeline) = {
            let pipeline_cache = gpu.ibl_bake_pipeline_cache.as_deref_mut().ok_or_else(|| {
                format!(
                    "IBL bake WGPU pass `{}` executor `{}` requires renderer IBL pipeline cache",
                    pass_name, executor_id
                )
            })?;
            let bind_group = create_ibl_bake_wgpu_bind_group(
                gpu.device,
                pipeline_cache.bind_group_layouts(),
                command,
                &params,
                &source_cubemap_view,
                &source_sampler,
                output.as_binding_resource(),
            )?;
            let pipeline = pipeline_cache.ensure_compute_pipeline(gpu.device, command);
            (bind_group, pipeline)
        };

        encode_ibl_bake_wgpu_compute_dispatch(gpu.encoder, command, &pipeline, &bind_group)?
    };

    let dispatch_record = RenderGraphComputeDispatchRecord::new(
        pass_name,
        executor_id,
        encoded.pipeline_label.clone(),
        IBL_BAKE_WORKGROUP_SIZE,
        encoded.dispatch_groups,
        vec![output_resource_name.to_string()],
    )
    .with_resource_accesses(resource_accesses);
    context
        .require_gpu()?
        .push_compute_dispatch_record(dispatch_record);

    Ok(encoded)
}

fn validate_dispatch_groups(command: &IblBakeWgpuCommandPlan) -> Result<(), String> {
    if command.dispatch_groups.iter().all(|group| *group > 0) {
        return Ok(());
    }
    Err(format!(
        "IBL bake command `{}` has invalid zero dispatch groups {:?}",
        command.pipeline_label, command.dispatch_groups
    ))
}

fn command_matches_context(
    command: &IblBakeWgpuCommandPlan,
    context: &RenderPassExecutionContext<'_>,
) -> bool {
    match command.kind {
        IblBakeComputeKernelKind::Pmrem { mip_level } => {
            context.executor_id.as_str() == IBL_BAKE_PMREM_EXECUTOR_ID
                && ibl_bake_pmrem_mip_from_pass_name(context.pass_name.as_str()) == Some(mip_level)
        }
        IblBakeComputeKernelKind::IrradianceSh9 => {
            context.executor_id.as_str() == IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID
                && context.pass_name == IBL_BAKE_IRRADIANCE_SH9_PASS
        }
        IblBakeComputeKernelKind::IrradianceCube => {
            context.executor_id.as_str() == IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID
                && context.pass_name == IBL_BAKE_IRRADIANCE_CUBE_PASS
        }
    }
}

fn require_context_access(
    context: &RenderPassExecutionContext<'_>,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> Result<(), String> {
    if context.declares_resource_name_access(resource_name, access) {
        return Ok(());
    }
    Err(format!(
        "IBL bake WGPU pass `{}` executor `{}` requires {:?} access to resource `{}`",
        context.pass_name, context.executor_id, access, resource_name
    ))
}

fn output_resource_name(command: &IblBakeWgpuCommandPlan) -> &'static str {
    match &command.output {
        IblBakeWgpuOutputPlan::StorageTexture { resource_name, .. } => resource_name,
        IblBakeWgpuOutputPlan::StorageBuffer { resource_name, .. } => resource_name,
    }
}

enum IblBakeResolvedOutputBinding {
    StorageTexture2DArray(wgpu::TextureView),
    StorageBuffer(wgpu::Buffer),
}

impl IblBakeResolvedOutputBinding {
    fn as_binding_resource(&self) -> IblBakeWgpuOutputBindingResource<'_> {
        match self {
            Self::StorageTexture2DArray(view) => {
                IblBakeWgpuOutputBindingResource::StorageTexture2DArray(view)
            }
            Self::StorageBuffer(buffer) => IblBakeWgpuOutputBindingResource::StorageBuffer(buffer),
        }
    }
}

fn resolve_output_binding(
    gpu: &RenderPassGpuExecutionContext<'_>,
    command: &IblBakeWgpuCommandPlan,
) -> Result<IblBakeResolvedOutputBinding, String> {
    match &command.output {
        IblBakeWgpuOutputPlan::StorageTexture {
            resource_name,
            view,
        } => {
            if let Some(resolver) = gpu.resource_resolver() {
                resolver.require_pass_resource_declaration_by_name(
                    resource_name,
                    RenderGraphResourceAccessKind::Write,
                )?;
            }
            let descriptor = view.to_wgpu_descriptor();
            gpu.resources
                .owned_texture_view_with_descriptor(resource_name, &descriptor)
                .map(IblBakeResolvedOutputBinding::StorageTexture2DArray)
        }
        IblBakeWgpuOutputPlan::StorageBuffer { resource_name, .. } => {
            if let Some(resolver) = gpu.resource_resolver() {
                resolver.require_pass_resource_declaration_by_name(
                    resource_name,
                    RenderGraphResourceAccessKind::Write,
                )?;
            }
            gpu.resources
                .buffer(resource_name)
                .cloned()
                .map(IblBakeResolvedOutputBinding::StorageBuffer)
                .ok_or_else(|| {
                    format!("render graph execution buffer resource `{resource_name}` is not bound")
                })
        }
    }
}

#[cfg(test)]
#[path = "ibl_bake_wgpu_dispatch/tests.rs"]
mod tests;
