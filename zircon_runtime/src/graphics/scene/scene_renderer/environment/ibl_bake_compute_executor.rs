use crate::core::framework::render::{
    EnvironmentExtract, IblBakeArtifactContents, IblBakeArtifactRequest,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphComputeDispatchRecord, RenderPassExecutionContext, RenderPassExecutorRegistration,
};
use crate::render_graph::{
    RenderGraphResourceAccessKind, RenderGraphResourceDesc, RenderGraphResourceKind,
};

use super::ibl_bake_graph_plan::{
    ibl_bake_pmrem_dispatch_groups, ibl_bake_pmrem_mip_from_pass_name,
    IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL,
    IBL_BAKE_IRRADIANCE_CUBE_RESOURCE, IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID,
    IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL, IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
    IBL_BAKE_PMREM_EXECUTOR_ID, IBL_BAKE_PMREM_PIPELINE_LABEL, IBL_BAKE_PMREM_RESOURCE,
    IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
};
use super::ibl_bake_wgpu_dispatch::record_ibl_bake_wgpu_pass_for_request;

const IBL_BAKE_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_compute_executor_registrations(
) -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new(IBL_BAKE_PMREM_EXECUTOR_ID, ibl_bake_pmrem_executor),
        RenderPassExecutorRegistration::new(
            IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID,
            ibl_bake_irradiance_sh9_executor,
        ),
        RenderPassExecutorRegistration::new(
            IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID,
            ibl_bake_irradiance_cube_executor,
        ),
    ]
}

fn ibl_bake_pmrem_executor(context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    record_ibl_bake_compute_dispatch(
        context,
        IBL_BAKE_PMREM_RESOURCE,
        IBL_BAKE_PMREM_PIPELINE_LABEL,
    )
}

fn ibl_bake_irradiance_sh9_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    record_ibl_bake_compute_dispatch(
        context,
        IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
        IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL,
    )
}

fn ibl_bake_irradiance_cube_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    record_ibl_bake_compute_dispatch(
        context,
        IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
        IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL,
    )
}

fn record_ibl_bake_compute_dispatch(
    context: &mut RenderPassExecutionContext<'_>,
    output_resource_name: &str,
    pipeline_label: &str,
) -> Result<(), String> {
    if context.gpu().is_some() {
        let request = ibl_bake_request_for_wgpu_context(context)?;
        record_ibl_bake_wgpu_pass_for_request(context, &request)?;
        return Ok(());
    }

    let dispatch = ibl_bake_compute_dispatch_record(context, output_resource_name, pipeline_label)?;
    context
        .require_gpu()?
        .push_compute_dispatch_record(dispatch);
    Ok(())
}

fn ibl_bake_request_for_wgpu_context(
    context: &RenderPassExecutionContext<'_>,
) -> Result<IblBakeArtifactRequest, String> {
    let environment = context
        .gpu()
        .map(|gpu| &gpu.frame_extract().environment)
        .ok_or_else(|| {
            format!(
                "IBL bake WGPU executor `{}` for pass `{}` requires current frame environment",
                context.executor_id, context.pass_name
            )
        })?;
    ibl_bake_request_from_frame_environment_and_graph_metadata(context, environment)
}

fn ibl_bake_request_from_frame_environment_and_graph_metadata(
    context: &RenderPassExecutionContext<'_>,
    environment: &EnvironmentExtract,
) -> Result<IblBakeArtifactRequest, String> {
    let required_contents = ibl_bake_required_contents_from_graph_metadata(context)?;
    environment
        .source_cubemap_ibl_bake_request(required_contents)
        .ok_or_else(|| {
            format!(
                "IBL bake WGPU executor `{}` for pass `{}` requires current frame source cubemap environment to provide bake key, face size, and mip count",
                context.executor_id, context.pass_name
            )
        })
}

fn ibl_bake_required_contents_from_graph_metadata(
    context: &RenderPassExecutionContext<'_>,
) -> Result<IblBakeArtifactContents, String> {
    let resolver = context.resource_resolver().ok_or_else(|| {
        format!(
            "IBL bake WGPU executor `{}` for pass `{}` requires render graph resource metadata to determine required artifact contents",
            context.executor_id, context.pass_name
        )
    })?;
    let mut contents = IblBakeArtifactContents::NONE;
    if resolver
        .resource_lifetime_by_name(IBL_BAKE_PMREM_RESOURCE)
        .is_some()
    {
        contents |= IblBakeArtifactContents::PMREM;
    }
    if resolver
        .resource_lifetime_by_name(IBL_BAKE_IRRADIANCE_SH9_RESOURCE)
        .is_some()
    {
        contents |= IblBakeArtifactContents::SH9;
    }
    if resolver
        .resource_lifetime_by_name(IBL_BAKE_IRRADIANCE_CUBE_RESOURCE)
        .is_some()
    {
        contents |= IblBakeArtifactContents::IEM;
    }
    if contents == IblBakeArtifactContents::NONE {
        return Err(format!(
            "IBL bake WGPU executor `{}` for pass `{}` requires at least one IBL bake output resource",
            context.executor_id, context.pass_name
        ));
    }
    Ok(contents)
}

fn ibl_bake_compute_dispatch_record(
    context: &RenderPassExecutionContext<'_>,
    output_resource_name: &str,
    pipeline_label: &str,
) -> Result<RenderGraphComputeDispatchRecord, String> {
    require_declared_access(
        context,
        IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
        RenderGraphResourceAccessKind::Read,
    )?;
    require_declared_access(
        context,
        output_resource_name,
        RenderGraphResourceAccessKind::Write,
    )?;
    let dispatch_groups = output_dispatch_groups(context, output_resource_name)?;

    Ok(RenderGraphComputeDispatchRecord::new(
        context.pass_name.clone(),
        context.executor_id.as_str().to_string(),
        pipeline_label,
        IBL_BAKE_WORKGROUP_SIZE,
        dispatch_groups,
        vec![output_resource_name.to_string()],
    )
    .with_resource_accesses(context.resources.clone()))
}

fn require_declared_access(
    context: &RenderPassExecutionContext<'_>,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> Result<(), String> {
    if context.declares_resource_name_access(resource_name, access) {
        return Ok(());
    }
    Err(format!(
        "IBL bake executor `{}` for pass `{}` requires {:?} access to resource `{}`",
        context.executor_id, context.pass_name, access, resource_name
    ))
}

fn output_dispatch_groups(
    context: &RenderPassExecutionContext<'_>,
    output_resource_name: &str,
) -> Result<[u32; 3], String> {
    let resolver = context.resource_resolver().ok_or_else(|| {
        format!(
            "IBL bake executor `{}` for pass `{}` requires render graph resource metadata",
            context.executor_id, context.pass_name
        )
    })?;
    let lifetime = resolver
        .resource_lifetime_by_name(output_resource_name)
        .ok_or_else(|| {
            format!(
                "IBL bake executor `{}` for pass `{}` references unknown output resource `{}`",
                context.executor_id, context.pass_name, output_resource_name
            )
        })?;

    match &lifetime.desc {
        RenderGraphResourceDesc::Texture(desc) => {
            if lifetime.kind != RenderGraphResourceKind::TransientTexture {
                return Err(format!(
                    "IBL bake output `{output_resource_name}` must be a transient texture"
                ));
            }
            if output_resource_name == IBL_BAKE_PMREM_RESOURCE {
                let mip_level = ibl_bake_pmrem_mip_from_pass_name(context.pass_name.as_str())
                    .ok_or_else(|| {
                        format!(
                            "IBL bake PMREM pass `{}` must include a `.mipN` suffix",
                            context.pass_name
                        )
                    })?;
                return Ok(ibl_bake_pmrem_dispatch_groups(
                    desc.width.min(desc.height),
                    desc.mip_levels,
                    mip_level,
                ));
            }
            Ok([
                div_ceil(desc.width, IBL_BAKE_WORKGROUP_SIZE[0]),
                div_ceil(desc.height, IBL_BAKE_WORKGROUP_SIZE[1]),
                desc.depth_or_array_layers(),
            ])
        }
        RenderGraphResourceDesc::Buffer(_) => {
            if lifetime.kind != RenderGraphResourceKind::TransientBuffer {
                return Err(format!(
                    "IBL bake output `{output_resource_name}` must be a transient buffer"
                ));
            }
            Ok([1, 1, 1])
        }
        RenderGraphResourceDesc::External => Err(format!(
            "IBL bake output `{output_resource_name}` must be transient, not external"
        )),
    }
}

const fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor
}

#[cfg(test)]
#[path = "tests/ibl_bake_compute_executor.rs"]
mod tests;
