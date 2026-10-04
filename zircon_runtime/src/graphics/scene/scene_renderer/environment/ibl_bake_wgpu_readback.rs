#[cfg(test)]
use crate::core::framework::render::IblBakeArtifactReadbackSections;
use crate::core::framework::render::{IblBakeArtifactContents, IblBakeArtifactDescriptor};
#[cfg(test)]
use crate::graphics::backend::read_ibl_bake_artifact_wgpu_sections;
use crate::graphics::backend::RenderBackend;
use crate::graphics::backend::{
    request_ibl_bake_artifact_wgpu_readback, IblBakeArtifactWgpuPendingReadback,
    IblBakeArtifactWgpuReadbackResources,
};
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::types::GraphicsError;
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphResourceAccessKind, RenderGraphResourceKind,
};
use std::ops::Range;

use super::environment_capture_gpu_target::EnvironmentCaptureGpuTarget;
use super::ibl_bake_graph_plan::{
    IBL_BAKE_IRRADIANCE_CUBE_RESOURCE, IBL_BAKE_IRRADIANCE_SH9_RESOURCE, IBL_BAKE_PMREM_RESOURCE,
};

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_wgpu_readback_resources_from_graph_resources<
    'a,
>(
    descriptor: IblBakeArtifactDescriptor,
    resources: &'a RenderGraphExecutionResources,
    graph: &CompiledRenderGraph,
) -> Result<IblBakeArtifactWgpuReadbackResources<'a>, String> {
    let mut readback = IblBakeArtifactWgpuReadbackResources::new(descriptor);
    let contents = descriptor.contents();

    if contents.contains(IblBakeArtifactContents::PMREM) {
        readback = readback.with_pmrem_texture(required_graph_texture(
            resources,
            graph,
            IBL_BAKE_PMREM_RESOURCE,
            RenderGraphResourceAccessKind::Write,
            "PMREM owned transient texture",
        )?);
    }

    if contents.contains(IblBakeArtifactContents::SH9) {
        let (buffer, range) = required_graph_buffer_binding(
            resources,
            graph,
            IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
            RenderGraphResourceAccessKind::Write,
            "SH9 storage buffer",
        )?;
        let size = range.end.checked_sub(range.start).ok_or_else(|| {
            format!(
                "IBL bake graph output `{IBL_BAKE_IRRADIANCE_SH9_RESOURCE}` has an inverted buffer range"
            )
        })?;
        let expected = descriptor
            .expected_irradiance_sh9_size_bytes()
            .ok_or_else(|| {
                format!(
                    "IBL bake graph output `{IBL_BAKE_IRRADIANCE_SH9_RESOURCE}` has no expected SH9 size"
                )
            })? as u64;
        if size != expected {
            return Err(format!(
                "IBL bake graph output `{IBL_BAKE_IRRADIANCE_SH9_RESOURCE}` range is {size} bytes, expected {expected}"
            ));
        }
        readback = readback.with_irradiance_sh9_buffer_range(buffer, range.start, size);
    }

    if contents.contains(IblBakeArtifactContents::IEM) {
        readback = readback.with_irradiance_cube_texture(required_graph_texture(
            resources,
            graph,
            IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
            RenderGraphResourceAccessKind::Write,
            "irradiance cube owned transient texture",
        )?);
    }

    Ok(readback)
}

#[cfg(test)]
pub(in crate::graphics::scene::scene_renderer) fn read_ibl_bake_artifact_wgpu_sections_from_graph_resources(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    descriptor: IblBakeArtifactDescriptor,
    resources: &RenderGraphExecutionResources,
    graph: &CompiledRenderGraph,
) -> Result<IblBakeArtifactReadbackSections, GraphicsError> {
    let readback =
        ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, resources, graph)
            .map_err(GraphicsError::BufferMap)?;
    read_ibl_bake_artifact_wgpu_sections(device, queue, readback)
}

pub(in crate::graphics::scene::scene_renderer) fn prepare_ibl_bake_artifact_wgpu_readback_from_graph_resources(
    backend: &RenderBackend,
    descriptor: IblBakeArtifactDescriptor,
    resources: &RenderGraphExecutionResources,
    graph: &CompiledRenderGraph,
) -> Result<IblBakeArtifactWgpuPendingReadback, GraphicsError> {
    let readback =
        ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, resources, graph)
            .map_err(GraphicsError::BufferMap)?;
    request_ibl_bake_artifact_wgpu_readback(backend, readback)
}

pub(in crate::graphics::scene::scene_renderer) fn prepare_ibl_bake_artifact_wgpu_readback_from_capture_target(
    backend: &RenderBackend,
    descriptor: IblBakeArtifactDescriptor,
    target: &EnvironmentCaptureGpuTarget,
) -> Result<IblBakeArtifactWgpuPendingReadback, GraphicsError> {
    let contents = descriptor.contents();
    let mut readback = IblBakeArtifactWgpuReadbackResources::new(descriptor);
    if contents.contains(IblBakeArtifactContents::PMREM) {
        readback = readback.with_pmrem_texture(target.pmrem_texture());
    }
    if contents.contains(IblBakeArtifactContents::SH9) {
        readback = readback.with_irradiance_sh9_buffer(target.sh9_buffer());
    }
    request_ibl_bake_artifact_wgpu_readback(backend, readback)
}

fn required_graph_texture<'a>(
    resources: &'a RenderGraphExecutionResources,
    graph: &CompiledRenderGraph,
    resource_name: &'static str,
    access_kind: RenderGraphResourceAccessKind,
    resource_role: &'static str,
) -> Result<&'a wgpu::Texture, String> {
    let mut binding = None;
    for pass in graph.passes() {
        if pass.culled {
            continue;
        }
        for (access_index, access) in pass.resources.iter().enumerate() {
            if access.name != resource_name || access.access != access_kind {
                continue;
            }
            if access.kind != RenderGraphResourceKind::TransientTexture {
                return Err(format!(
                    "IBL bake graph output `{resource_name}` is not a transient texture resource"
                ));
            }
            let access_id = graph.access_id_at(pass.id, access_index).ok_or_else(|| {
                format!("IBL bake graph output `{resource_name}` has no compiled access identity")
            })?;
            let physical_allocation = resources
                .transient_physical_allocation_for_access(access_id)
                .ok_or_else(|| {
                    format!(
                        "IBL bake graph output `{resource_name}` access {access_id:?} has no physical allocation identity"
                    )
                })?;
            let texture = resources.transient_texture_for_access(access_id)?;
            match binding {
                Some((_, expected_allocation)) if expected_allocation != physical_allocation => {
                    return Err(format!(
                        "IBL bake graph output `{resource_name}` live writes resolve to different physical allocations"
                    ));
                }
                Some(_) => {}
                None => binding = Some((texture, physical_allocation)),
            }
        }
    }
    binding
        .map(|(texture, _)| texture)
        .ok_or_else(|| missing_readback_resource(resource_name, resource_role))
}

fn required_graph_buffer_binding<'a>(
    resources: &'a RenderGraphExecutionResources,
    graph: &CompiledRenderGraph,
    resource_name: &'static str,
    access_kind: RenderGraphResourceAccessKind,
    resource_role: &'static str,
) -> Result<(&'a wgpu::Buffer, Range<wgpu::BufferAddress>), String> {
    let mut binding = None;
    for pass in graph.passes() {
        if pass.culled {
            continue;
        }
        for (access_index, access) in pass.resources.iter().enumerate() {
            if access.name != resource_name || access.access != access_kind {
                continue;
            }
            let access_id = graph.access_id_at(pass.id, access_index).ok_or_else(|| {
                format!("IBL bake graph output `{resource_name}` has no compiled access identity")
            })?;
            let resolved = match access.kind {
                RenderGraphResourceKind::TransientBuffer => {
                    resources.transient_buffer_binding_for_access(access_id)
                }
                RenderGraphResourceKind::External => {
                    resources.external_buffer_binding_for_access(access_id)
                }
                _ => Err(format!(
                    "IBL bake graph output `{resource_name}` is not a buffer resource"
                )),
            }?;
            if binding.replace(resolved).is_some() {
                return Err(format!(
                    "IBL bake graph output `{resource_name}` has multiple live write bindings"
                ));
            }
        }
    }
    binding.ok_or_else(|| missing_readback_resource(resource_name, resource_role))
}

fn missing_readback_resource(resource_name: &'static str, resource_role: &'static str) -> String {
    format!("missing required IBL bake graph readback resource `{resource_name}` ({resource_role})")
}

#[cfg(test)]
#[path = "tests/ibl_bake_wgpu_readback.rs"]
mod tests;
