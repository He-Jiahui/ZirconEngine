use std::sync::Arc;

use crate::core::framework::render::{
    RenderMeshLodSelection, RenderMeshStaticState, RendererCommon,
};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::{
    GpuMaterialUniformResource, GpuMeshResource, MaterialDisabledPasses, PipelineKey,
};

use super::super::mesh_draw::{
    MaterialTextureSet, MeshDraw, MeshDrawGeometrySource, VirtualGeometrySubmissionDetail,
};

const MATERIAL_TEXTURE_BINDING_COUNT: usize = 6;
const MATERIAL_BIND_GROUP_COUNT: usize = 2;
const MATERIAL_BIND_GROUP_ENTRY_COUNT: usize = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MaterialBindingBuildProfile {
    residual_draw_count: usize,
    sampler_variant_query_count: usize,
    bind_group_creation_count: usize,
    bind_group_entry_projection_count: usize,
    override_uniform_buffer_creation_count: usize,
}

impl MaterialBindingBuildProfile {
    const fn for_residual_draws(
        residual_draw_count: usize,
        override_uniform_buffer_creation_count: usize,
    ) -> Self {
        let bind_group_creation_count =
            residual_draw_count.saturating_mul(MATERIAL_BIND_GROUP_COUNT);
        Self {
            residual_draw_count,
            sampler_variant_query_count: residual_draw_count
                .saturating_mul(MATERIAL_TEXTURE_BINDING_COUNT),
            bind_group_creation_count,
            bind_group_entry_projection_count: bind_group_creation_count
                .saturating_mul(MATERIAL_BIND_GROUP_ENTRY_COUNT),
            override_uniform_buffer_creation_count,
        }
    }
}

pub(super) fn record_material_binding_build_profile(
    residual_draw_count: usize,
    override_uniform_buffer_creation_count: usize,
) {
    debug_assert!(override_uniform_buffer_creation_count <= residual_draw_count);
    let _profile = MaterialBindingBuildProfile::for_residual_draws(
        residual_draw_count,
        override_uniform_buffer_creation_count,
    );
    crate::profile_counter!(
        "render",
        "material.binding.residual_draw_count",
        _profile.residual_draw_count
    );
    crate::profile_counter!(
        "render",
        "material.binding.sampler_variant_query_count",
        _profile.sampler_variant_query_count
    );
    crate::profile_counter!(
        "render",
        "material.binding.bind_group_creation_count",
        _profile.bind_group_creation_count
    );
    crate::profile_counter!(
        "render",
        "material.binding.entry_projection_count",
        _profile.bind_group_entry_projection_count
    );
    crate::profile_counter!(
        "render",
        "material.binding.override_uniform_buffer_creation_count",
        _profile.override_uniform_buffer_creation_count
    );
}

pub(super) fn create_mesh_draw(
    device: &wgpu::Device,
    material_texture_layout: &wgpu::BindGroupLayout,
    mesh: Arc<GpuMeshResource>,
    geometry_source: MeshDrawGeometrySource,
    mobility: Mobility,
    source_entity: EntityId,
    material_id: ResourceId,
    stable_instance_key: u64,
    source_draw_ordinal: u32,
    static_state: RenderMeshStaticState,
    mut material_textures: MaterialTextureSet,
    material_uniform: Arc<GpuMaterialUniformResource>,
    standard_material_uniform: Arc<GpuMaterialUniformResource>,
    pipeline_key: PipelineKey,
    common: &RendererCommon,
    disabled_passes: MaterialDisabledPasses,
    taa_reactive_mask_strength: f32,
    half_resolution_transparency: bool,
    has_previous_velocity_transform: bool,
    mesh_lod: Option<RenderMeshLodSelection>,
    skinned: bool,
    has_skinned_joint_palette_upload: bool,
    has_previous_skinned_joint_palette_upload: bool,
    previous_skinned_gpu_source: Option<Arc<GpuMeshResource>>,
    skinned_gpu_source: Option<Arc<GpuMeshResource>>,
    skinned_gpu_source_uses_cpu_morphed_source: bool,
    skinned_gpu_skinning_enabled: bool,
    first_index: u32,
    draw_index_count: u32,
    indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_args_offset: u64,
    virtual_geometry_submission_detail: Option<VirtualGeometrySubmissionDetail>,
) -> MeshDraw {
    material_textures.prepare_sampler_variants(device);
    let material_bind_group = create_material_bind_group(
        device,
        material_texture_layout,
        &material_textures,
        &material_uniform,
        "zircon-material-set-bind-group",
    );
    let standard_material_bind_group = create_material_bind_group(
        device,
        material_texture_layout,
        &material_textures,
        &standard_material_uniform,
        "zircon-standard-material-set-bind-group",
    );

    MeshDraw::new(
        mesh,
        geometry_source,
        mobility,
        source_entity,
        material_id,
        stable_instance_key,
        source_draw_ordinal,
        static_state,
        first_index,
        draw_index_count,
        indirect_args_buffer,
        indirect_args_offset,
        virtual_geometry_submission_detail,
        material_textures,
        material_bind_group,
        standard_material_bind_group,
        material_uniform,
        standard_material_uniform,
        pipeline_key,
        common.cast_shadows,
        disabled_passes,
        taa_reactive_mask_strength,
        half_resolution_transparency,
        has_previous_velocity_transform,
        mesh_lod,
        skinned,
        has_skinned_joint_palette_upload,
        has_previous_skinned_joint_palette_upload,
        previous_skinned_gpu_source,
        skinned_gpu_source,
        skinned_gpu_source_uses_cpu_morphed_source,
        skinned_gpu_skinning_enabled,
    )
}

fn create_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    material_textures: &MaterialTextureSet,
    material_uniform: &GpuMaterialUniformResource,
    label: &'static str,
) -> wgpu::BindGroup {
    let entries = [
        wgpu::BindGroupEntry {
            binding: 0,
            resource: material_uniform.binding_resource(),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(material_textures.base_color.view()),
        },
        wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::Sampler(material_textures.base_color.sampler()),
        },
        wgpu::BindGroupEntry {
            binding: 3,
            resource: wgpu::BindingResource::TextureView(material_textures.normal.view()),
        },
        wgpu::BindGroupEntry {
            binding: 4,
            resource: wgpu::BindingResource::Sampler(material_textures.normal.sampler()),
        },
        wgpu::BindGroupEntry {
            binding: 5,
            resource: wgpu::BindingResource::TextureView(
                material_textures.metallic_roughness.view(),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 6,
            resource: wgpu::BindingResource::Sampler(
                material_textures.metallic_roughness.sampler(),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 7,
            resource: wgpu::BindingResource::TextureView(material_textures.occlusion.view()),
        },
        wgpu::BindGroupEntry {
            binding: 8,
            resource: wgpu::BindingResource::Sampler(material_textures.occlusion.sampler()),
        },
        wgpu::BindGroupEntry {
            binding: 9,
            resource: wgpu::BindingResource::TextureView(material_textures.emissive.view()),
        },
        wgpu::BindGroupEntry {
            binding: 10,
            resource: wgpu::BindingResource::Sampler(material_textures.emissive.sampler()),
        },
        wgpu::BindGroupEntry {
            binding: 11,
            resource: wgpu::BindingResource::TextureView(material_textures.clearcoat_normal.view()),
        },
        wgpu::BindGroupEntry {
            binding: 12,
            resource: wgpu::BindingResource::Sampler(material_textures.clearcoat_normal.sampler()),
        },
    ];
    debug_assert_eq!(entries.len(), MATERIAL_BIND_GROUP_ENTRY_COUNT);
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

#[cfg(test)]
mod tests {
    use super::MaterialBindingBuildProfile;

    #[test]
    fn material_binding_profile_matches_the_fixed_draw_abi() {
        assert_eq!(
            MaterialBindingBuildProfile::for_residual_draws(1, 0),
            MaterialBindingBuildProfile {
                residual_draw_count: 1,
                sampler_variant_query_count: 6,
                bind_group_creation_count: 2,
                bind_group_entry_projection_count: 26,
                override_uniform_buffer_creation_count: 0,
            }
        );
    }

    #[test]
    fn material_binding_profile_exposes_linear_draw_amplification() {
        assert_eq!(
            MaterialBindingBuildProfile::for_residual_draws(10_000, 375),
            MaterialBindingBuildProfile {
                residual_draw_count: 10_000,
                sampler_variant_query_count: 60_000,
                bind_group_creation_count: 20_000,
                bind_group_entry_projection_count: 260_000,
                override_uniform_buffer_creation_count: 375,
            }
        );
    }
}
