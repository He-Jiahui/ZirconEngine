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
mod tests {
    use super::*;
    use crate::core::framework::render::{
        IblBakeArtifactRequest, ProceduralSkyParams, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
    };
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::scene_renderer::graph_execution::TransientResourcePool;
    use crate::render_graph::{
        PassFlags, QueueLane, RenderGraphBufferRange, RenderGraphBuilder,
        RenderGraphExternalResourceBinding, RenderGraphResourceAccessIntent,
        RenderGraphResourceAccessRange, RenderGraphShaderStages,
    };
    use crate::rhi::{BufferDesc, BufferUsage};

    #[test]
    fn readback_resources_resolve_pmrem_sh9_iem_from_materialized_graph() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let request = request(16, 5, IblBakeArtifactContents::PMREM_SH9_IEM);
        let (mut resources, graph) = materialized_ibl_bake_resources(&backend, &request);
        let descriptor = descriptor_for(&request);

        let readback =
            ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, &resources, &graph)
                .expect("full IBL bake graph resources should resolve for readback");

        assert_eq!(readback.descriptor(), descriptor);
        assert!(readback.requires_pmrem_texture());
        assert!(readback.requires_irradiance_sh9_buffer());
        assert!(readback.requires_irradiance_cube_texture());
        assert_eq!(
            resources
                .owned_texture_mip_level_count(IBL_BAKE_IRRADIANCE_CUBE_RESOURCE)
                .expect("IEM texture mip count"),
            1
        );
        assert!(resources.owned_texture(IBL_BAKE_PMREM_RESOURCE).is_some());
        assert!(resources.buffer(IBL_BAKE_IRRADIANCE_SH9_RESOURCE).is_some());
        assert!(resources
            .owned_texture(IBL_BAKE_IRRADIANCE_CUBE_RESOURCE)
            .is_some());
        resources.release_transient_backings_into_pool(&mut Default::default());
    }

    #[test]
    fn readback_resources_allow_sh9_only_graph_outputs() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let request = request(16, 5, IblBakeArtifactContents::SH9);
        let (resources, graph) = materialized_ibl_bake_resources(&backend, &request);
        let descriptor = descriptor_for(&request);

        let readback =
            ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, &resources, &graph)
                .expect("SH9-only IBL bake graph resources should resolve for readback");

        assert_eq!(readback.descriptor(), descriptor);
        assert!(!readback.requires_pmrem_texture());
        assert!(readback.requires_irradiance_sh9_buffer());
        assert!(!readback.requires_irradiance_cube_texture());
        assert!(resources.owned_texture(IBL_BAKE_PMREM_RESOURCE).is_none());
        assert!(resources.buffer(IBL_BAKE_IRRADIANCE_SH9_RESOURCE).is_some());
    }

    #[test]
    fn graph_buffer_binding_preserves_nonzero_external_sh9_window() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let descriptor = descriptor(16, 5, IblBakeArtifactContents::SH9);
        let expected_size = descriptor.expected_irradiance_sh9_size_bytes().unwrap() as u64;
        let mut builder = RenderGraphBuilder::new("ibl-bake-readback-external-window");
        let buffer = builder.import_present_external_buffer_with_binding(
            IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
            BufferDesc::new(
                IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
                512,
                BufferUsage::STORAGE | BufferUsage::COPY_SRC,
            ),
            RenderGraphExternalResourceBinding::required_buffer(),
        );
        let pass = builder.add_pass("ibl-bake-external-sh9-writer", QueueLane::AsyncCompute);
        builder
            .access_external(
                pass,
                buffer,
                RenderGraphResourceAccessKind::Write,
                RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(
                    64,
                    Some(expected_size),
                )),
                RenderGraphResourceAccessIntent::storage_buffer_read_write(
                    RenderGraphShaderStages::COMPUTE,
                ),
                None,
            )
            .unwrap();
        builder
            .set_pass_flags(
                pass,
                PassFlags {
                    has_side_effects: true,
                    ..PassFlags::default()
                },
            )
            .unwrap();
        let graph = builder.compile().unwrap();
        let native = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ibl-bake-readback-external-window"),
            size: 512,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let mut resources = RenderGraphExecutionResources::new();
        resources.insert_buffer(IBL_BAKE_IRRADIANCE_SH9_RESOURCE, native);
        resources
            .materialize_external_access_bindings(&graph)
            .unwrap();

        let (_, range) = required_graph_buffer_binding(
            &resources,
            &graph,
            IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
            RenderGraphResourceAccessKind::Write,
            "SH9 storage buffer",
        )
        .unwrap();
        let readback =
            ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, &resources, &graph)
                .expect("non-zero external SH9 window should form a backend readback packet");

        assert_eq!(range, 64..64 + expected_size);
        assert!(readback.requires_irradiance_sh9_buffer());
    }

    #[test]
    fn missing_readback_resource_names_the_required_graph_resource() {
        let resources = RenderGraphExecutionResources::new();
        let descriptor = descriptor(16, 5, IblBakeArtifactContents::PMREM_SH9_IEM);
        let graph = RenderGraphBuilder::new("ibl-bake-readback-empty")
            .compile()
            .unwrap();

        let error = match ibl_bake_wgpu_readback_resources_from_graph_resources(
            descriptor, &resources, &graph,
        ) {
            Ok(_) => panic!("missing PMREM graph output should fail before backend readback"),
            Err(error) => error,
        };

        assert!(error.contains(IBL_BAKE_PMREM_RESOURCE));
        assert!(error.contains("PMREM owned transient texture"));
    }

    #[test]
    fn iem_readback_resource_descriptor_uses_single_irradiance_cube_mip() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let request = request(32, 6, IblBakeArtifactContents::PMREM_SH9_IEM);
        let (resources, _graph) = materialized_ibl_bake_resources(&backend, &request);

        let iem_desc = resources
            .owned_texture_desc(IBL_BAKE_IRRADIANCE_CUBE_RESOURCE)
            .expect("IEM graph texture descriptor");

        assert_eq!(iem_desc.width, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE);
        assert_eq!(iem_desc.height, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE);
        assert_eq!(iem_desc.depth, 6);
        assert_eq!(iem_desc.mip_levels, 1);
    }

    #[test]
    fn readback_descriptor_preserves_independent_source_and_pmrem_layouts() {
        let request = request(16, 5, IblBakeArtifactContents::PMREM_SH9_IEM);
        let descriptor = descriptor_for(&request);

        assert_ne!(request.source_face_size(), request.pmrem_face_size());
        assert_ne!(request.source_mip_count(), request.pmrem_mip_count());
        assert!(descriptor.is_current_for(&request));
    }

    fn materialized_ibl_bake_resources(
        backend: &RenderBackend,
        request: &IblBakeArtifactRequest,
    ) -> (
        RenderGraphExecutionResources,
        crate::render_graph::CompiledRenderGraph,
    ) {
        let mut builder = RenderGraphBuilder::new("ibl-bake-readback-test");
        super::super::ibl_bake_graph_plan::append_ibl_bake_artifact_graph_plan(
            &mut builder,
            request,
        )
        .expect("IBL bake graph plan should append");
        let graph = builder.compile().expect("IBL bake graph should compile");
        let mut resources = RenderGraphExecutionResources::new();
        let mut transient_pool = TransientResourcePool::default();
        transient_pool.begin_frame(backend.device_profile());
        resources
            .materialize_transient_resources_with_pool(
                &backend.device,
                backend.device_profile(),
                &graph,
                &mut transient_pool,
            )
            .expect("IBL bake transient outputs should materialize");
        (resources, graph)
    }

    fn request(
        face_size: u32,
        mip_count: u32,
        contents: IblBakeArtifactContents,
    ) -> IblBakeArtifactRequest {
        IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            face_size,
            mip_count,
        )
        .with_required_contents(contents)
    }

    fn descriptor_for(request: &IblBakeArtifactRequest) -> IblBakeArtifactDescriptor {
        IblBakeArtifactDescriptor::current_for_request(request)
    }

    fn descriptor(
        face_size: u32,
        mip_count: u32,
        contents: IblBakeArtifactContents,
    ) -> IblBakeArtifactDescriptor {
        descriptor_for(&request(face_size, mip_count, contents))
    }
}
