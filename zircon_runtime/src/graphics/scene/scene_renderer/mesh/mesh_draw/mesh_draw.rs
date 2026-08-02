use std::sync::Arc;

use crate::core::framework::render::{
    CastShadowsMode, PrimitiveRelevance, RenderMeshLodSelection, RenderMeshStaticState,
};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::graphics::scene::resources::{
    GpuMaterialUniformResource, GpuMeshResource, MaterialDisabledPasses, PipelineKey,
};

use super::geometry_source::MeshDrawGeometrySource;
use super::material_texture_set::MaterialTextureSet;
use super::virtual_geometry_submission_detail::VirtualGeometrySubmissionDetail;
use super::MeshCommandSortInput;

pub(crate) struct MeshDraw {
    pub(super) mesh: Arc<GpuMeshResource>,
    pub(super) geometry_source: MeshDrawGeometrySource,
    pub(super) mobility: Mobility,
    pub(super) source_entity: EntityId,
    pub(super) stable_instance_key: u64,
    pub(super) source_draw_ordinal: u32,
    pub(super) static_state: RenderMeshStaticState,
    pub(super) first_index: u32,
    pub(super) draw_index_count: u32,
    pub(super) indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) indirect_args_offset: u64,
    pub(super) virtual_geometry_submission_key: Option<(u64, u32)>,
    pub(super) virtual_geometry_submission_detail: Option<VirtualGeometrySubmissionDetail>,
    pub(super) material_textures: MaterialTextureSet,
    pub(super) material_bind_group: wgpu::BindGroup,
    pub(super) standard_material_bind_group: wgpu::BindGroup,
    pub(super) material_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) standard_material_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) pipeline_key: PipelineKey,
    pub(super) cast_shadows: CastShadowsMode,
    pub(super) disabled_passes: MaterialDisabledPasses,
    pub(super) taa_reactive_mask_strength: f32,
    pub(super) gpu_scene_bind_group: Option<wgpu::BindGroup>,
    pub(super) gpu_scene_instance_span: Option<(u32, u32)>,
    pub(super) primitive_relevance: Option<PrimitiveRelevance>,
    pub(super) main_view_visible: bool,
    pub(super) shadow_view_visible: bool,
    pub(super) has_previous_velocity_transform: bool,
    pub(super) mesh_lod: Option<RenderMeshLodSelection>,
    pub(super) skinned: bool,
    pub(super) skinned_joint_palette_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) previous_skinned_joint_palette_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) previous_skinned_gpu_source: Option<Arc<GpuMeshResource>>,
    pub(super) command_sort_input: MeshCommandSortInput,
    // Retains the source mesh that allowed this draw to enter the shader-skinning
    // path. Draws without this source remain CPU-skinned dynamic fallbacks.
    pub(super) skinned_gpu_source: Option<Arc<GpuMeshResource>>,
    pub(super) skinned_gpu_source_uses_cpu_morphed_source: bool,
    pub(super) skinned_gpu_skinning_enabled: bool,
}

impl MeshDraw {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene) fn new(
        mesh: Arc<GpuMeshResource>,
        geometry_source: MeshDrawGeometrySource,
        mobility: Mobility,
        source_entity: EntityId,
        stable_instance_key: u64,
        source_draw_ordinal: u32,
        static_state: RenderMeshStaticState,
        first_index: u32,
        draw_index_count: u32,
        indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
        indirect_args_offset: u64,
        virtual_geometry_submission_detail: Option<VirtualGeometrySubmissionDetail>,
        material_textures: MaterialTextureSet,
        material_bind_group: wgpu::BindGroup,
        standard_material_bind_group: wgpu::BindGroup,
        material_uniform: Arc<GpuMaterialUniformResource>,
        standard_material_uniform: Arc<GpuMaterialUniformResource>,
        pipeline_key: PipelineKey,
        cast_shadows: CastShadowsMode,
        disabled_passes: MaterialDisabledPasses,
        taa_reactive_mask_strength: f32,
        gpu_scene_bind_group: Option<wgpu::BindGroup>,
        has_previous_velocity_transform: bool,
        mesh_lod: Option<RenderMeshLodSelection>,
        skinned: bool,
        skinned_joint_palette_buffer: Option<Arc<wgpu::Buffer>>,
        previous_skinned_joint_palette_buffer: Option<Arc<wgpu::Buffer>>,
        previous_skinned_gpu_source: Option<Arc<GpuMeshResource>>,
        skinned_gpu_source: Option<Arc<GpuMeshResource>>,
        skinned_gpu_source_uses_cpu_morphed_source: bool,
        skinned_gpu_skinning_enabled: bool,
    ) -> Self {
        Self {
            mesh,
            geometry_source,
            mobility,
            source_entity,
            stable_instance_key,
            source_draw_ordinal,
            static_state,
            first_index,
            draw_index_count,
            indirect_args_buffer,
            indirect_args_offset,
            virtual_geometry_submission_key: virtual_geometry_submission_detail
                .map(|detail| (detail.entity(), detail.page_id())),
            virtual_geometry_submission_detail,
            material_textures,
            material_bind_group,
            standard_material_bind_group,
            material_uniform,
            standard_material_uniform,
            pipeline_key,
            cast_shadows,
            disabled_passes,
            taa_reactive_mask_strength,
            gpu_scene_bind_group,
            gpu_scene_instance_span: None,
            primitive_relevance: None,
            main_view_visible: true,
            shadow_view_visible: true,
            has_previous_velocity_transform,
            mesh_lod,
            skinned,
            skinned_joint_palette_buffer,
            previous_skinned_joint_palette_buffer,
            previous_skinned_gpu_source,
            command_sort_input: MeshCommandSortInput::new(0.0, source_entity),
            skinned_gpu_source,
            skinned_gpu_source_uses_cpu_morphed_source,
            skinned_gpu_skinning_enabled,
        }
    }

    pub(crate) fn has_previous_velocity_transform(&self) -> bool {
        self.has_previous_velocity_transform
    }

    pub(crate) fn source_entity(&self) -> EntityId {
        self.source_entity
    }

    pub(crate) fn stable_instance_key(&self) -> u64 {
        self.stable_instance_key
    }

    pub(crate) fn source_draw_ordinal(&self) -> u32 {
        self.source_draw_ordinal
    }

    pub(crate) fn static_state(&self) -> RenderMeshStaticState {
        self.static_state
    }

    pub(crate) fn with_gpu_scene_instance_span(
        mut self,
        first_instance_index: u32,
        instance_count: u32,
    ) -> Self {
        debug_assert!(instance_count > 0);
        self.gpu_scene_instance_span = Some((first_instance_index, instance_count));
        self
    }

    pub(crate) fn with_visibility(
        mut self,
        relevance: PrimitiveRelevance,
        main_view_visible: bool,
        shadow_view_visible: bool,
    ) -> Self {
        self.primitive_relevance = Some(relevance);
        self.main_view_visible = main_view_visible;
        self.shadow_view_visible = shadow_view_visible;
        self
    }

    pub(crate) fn with_command_sort_input(
        mut self,
        command_sort_input: MeshCommandSortInput,
    ) -> Self {
        self.command_sort_input = command_sort_input;
        self
    }
}
