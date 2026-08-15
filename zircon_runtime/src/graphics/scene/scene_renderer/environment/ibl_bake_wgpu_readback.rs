use crate::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactReadbackSections,
};
use crate::graphics::backend::{
    prepare_ibl_bake_artifact_wgpu_readback, read_ibl_bake_artifact_wgpu_sections,
    IblBakeArtifactWgpuPendingReadback, IblBakeArtifactWgpuReadbackResources,
};
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::types::GraphicsError;

use super::ibl_bake_graph_plan::{
    IBL_BAKE_IRRADIANCE_CUBE_RESOURCE, IBL_BAKE_IRRADIANCE_SH9_RESOURCE, IBL_BAKE_PMREM_RESOURCE,
};

pub(in crate::graphics::scene::scene_renderer) fn ibl_bake_wgpu_readback_resources_from_graph_resources<
    'a,
>(
    descriptor: IblBakeArtifactDescriptor,
    resources: &'a RenderGraphExecutionResources,
) -> Result<IblBakeArtifactWgpuReadbackResources<'a>, String> {
    let mut readback = IblBakeArtifactWgpuReadbackResources::new(descriptor);
    let contents = descriptor.contents();

    if contents.contains(IblBakeArtifactContents::PMREM) {
        readback = readback.with_pmrem_texture(required_owned_texture(
            resources,
            IBL_BAKE_PMREM_RESOURCE,
            "PMREM owned transient texture",
        )?);
    }

    if contents.contains(IblBakeArtifactContents::SH9) {
        readback = readback.with_irradiance_sh9_buffer(required_buffer(
            resources,
            IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
            "SH9 storage buffer",
        )?);
    }

    if contents.contains(IblBakeArtifactContents::IEM) {
        readback = readback.with_irradiance_cube_texture(required_owned_texture(
            resources,
            IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
            "irradiance cube owned transient texture",
        )?);
    }

    Ok(readback)
}

pub(in crate::graphics::scene::scene_renderer) fn read_ibl_bake_artifact_wgpu_sections_from_graph_resources(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    descriptor: IblBakeArtifactDescriptor,
    resources: &RenderGraphExecutionResources,
) -> Result<IblBakeArtifactReadbackSections, GraphicsError> {
    let readback = ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, resources)
        .map_err(GraphicsError::BufferMap)?;
    read_ibl_bake_artifact_wgpu_sections(device, queue, readback)
}

pub(in crate::graphics::scene::scene_renderer) fn prepare_ibl_bake_artifact_wgpu_readback_from_graph_resources(
    device: &wgpu::Device,
    descriptor: IblBakeArtifactDescriptor,
    resources: &RenderGraphExecutionResources,
) -> Result<IblBakeArtifactWgpuPendingReadback, GraphicsError> {
    let readback = ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, resources)
        .map_err(GraphicsError::BufferMap)?;
    prepare_ibl_bake_artifact_wgpu_readback(device, readback)
}

fn required_owned_texture<'a>(
    resources: &'a RenderGraphExecutionResources,
    resource_name: &'static str,
    resource_role: &'static str,
) -> Result<&'a wgpu::Texture, String> {
    resources
        .owned_texture(resource_name)
        .ok_or_else(|| missing_readback_resource(resource_name, resource_role))
}

fn required_buffer<'a>(
    resources: &'a RenderGraphExecutionResources,
    resource_name: &'static str,
    resource_role: &'static str,
) -> Result<&'a wgpu::Buffer, String> {
    resources
        .buffer(resource_name)
        .ok_or_else(|| missing_readback_resource(resource_name, resource_role))
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
    use crate::render_graph::RenderGraphBuilder;

    #[test]
    fn readback_resources_resolve_pmrem_sh9_iem_from_materialized_graph() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let request = request(16, 5, IblBakeArtifactContents::PMREM_SH9_IEM);
        let mut resources = materialized_ibl_bake_resources(&backend, &request);
        let descriptor = descriptor_for(&request);

        let readback =
            ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, &resources)
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
        let resources = materialized_ibl_bake_resources(&backend, &request);
        let descriptor = descriptor_for(&request);

        let readback =
            ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, &resources)
                .expect("SH9-only IBL bake graph resources should resolve for readback");

        assert_eq!(readback.descriptor(), descriptor);
        assert!(!readback.requires_pmrem_texture());
        assert!(readback.requires_irradiance_sh9_buffer());
        assert!(!readback.requires_irradiance_cube_texture());
        assert!(resources.owned_texture(IBL_BAKE_PMREM_RESOURCE).is_none());
        assert!(resources.buffer(IBL_BAKE_IRRADIANCE_SH9_RESOURCE).is_some());
    }

    #[test]
    fn missing_readback_resource_names_the_required_graph_resource() {
        let resources = RenderGraphExecutionResources::new();
        let descriptor = descriptor(16, 5, IblBakeArtifactContents::PMREM_SH9_IEM);

        let error =
            match ibl_bake_wgpu_readback_resources_from_graph_resources(descriptor, &resources) {
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
        let resources = materialized_ibl_bake_resources(&backend, &request);

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
    ) -> RenderGraphExecutionResources {
        let mut builder = RenderGraphBuilder::new("ibl-bake-readback-test");
        super::super::ibl_bake_graph_plan::append_ibl_bake_artifact_graph_plan(
            &mut builder,
            request,
        )
        .expect("IBL bake graph plan should append");
        let graph = builder.compile().expect("IBL bake graph should compile");
        let mut resources = RenderGraphExecutionResources::new();
        let mut transient_pool = TransientResourcePool::default();
        transient_pool.begin_frame();
        resources
            .materialize_transient_resources_with_pool(&backend.device, &graph, &mut transient_pool)
            .expect("IBL bake transient outputs should materialize");
        resources
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
