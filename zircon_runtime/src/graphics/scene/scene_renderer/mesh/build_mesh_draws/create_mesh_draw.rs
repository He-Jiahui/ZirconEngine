use std::sync::Arc;

use crate::core::framework::render::{RenderMeshLodSelection, RenderMeshStaticState};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::resources::{GpuMaterialUniformResource, GpuMeshResource, PipelineKey};

use super::super::mesh_draw::{
    MaterialTextureSet, MeshDraw, MeshDrawGeometrySource, VirtualGeometrySubmissionDetail,
};
use super::super::skinning::SkinnedMeshJointPaletteUniform;

pub(super) fn create_mesh_draw(
    device: &wgpu::Device,
    gpu_scene: &GpuScene,
    material_texture_layout: &wgpu::BindGroupLayout,
    mesh: Arc<GpuMeshResource>,
    geometry_source: MeshDrawGeometrySource,
    mobility: Mobility,
    source_entity: EntityId,
    source_draw_ordinal: u32,
    static_state: RenderMeshStaticState,
    material_textures: MaterialTextureSet,
    material_uniform: Arc<GpuMaterialUniformResource>,
    standard_material_uniform: Arc<GpuMaterialUniformResource>,
    pipeline_key: PipelineKey,
    cast_shadows: bool,
    taa_reactive_mask_strength: f32,
    has_previous_velocity_transform: bool,
    mesh_lod: Option<RenderMeshLodSelection>,
    skinned: bool,
    skinned_joint_palette: Option<SkinnedMeshJointPaletteUniform>,
    previous_skinned_joint_palette: Option<SkinnedMeshJointPaletteUniform>,
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
    let skinned_joint_palette_buffer =
        skinned_joint_palette.map(|uniform| uniform.create_buffer(device));
    let previous_skinned_joint_palette_buffer =
        previous_skinned_joint_palette.map(|uniform| uniform.create_buffer(device));
    let gpu_scene_bind_group = (skinned_joint_palette_buffer.is_some()
        || previous_skinned_joint_palette_buffer.is_some())
    .then(|| {
        gpu_scene.create_scene_bind_group_for_palettes(
            device,
            skinned_joint_palette_buffer.as_deref(),
            previous_skinned_joint_palette_buffer.as_deref(),
        )
    });
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
        cast_shadows,
        taa_reactive_mask_strength,
        gpu_scene_bind_group,
        has_previous_velocity_transform,
        mesh_lod,
        skinned,
        skinned_joint_palette_buffer,
        previous_skinned_joint_palette_buffer,
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
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(material_textures.base_color.view()),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(material_textures.base_color.sampler()),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(material_textures.normal.view()),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Sampler(material_textures.normal.sampler()),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(
                    material_textures.metallic_roughness.view(),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(
                    material_textures.metallic_roughness.sampler(),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(material_textures.occlusion.view()),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: wgpu::BindingResource::Sampler(material_textures.occlusion.sampler()),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: wgpu::BindingResource::TextureView(material_textures.emissive.view()),
            },
            wgpu::BindGroupEntry {
                binding: 9,
                resource: wgpu::BindingResource::Sampler(material_textures.emissive.sampler()),
            },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: material_uniform.binding_resource(),
            },
        ],
    })
}
