use super::{
    DrawInstanceSource, MeshBindHandle, MeshDrawArgs, MeshDrawCommand, MeshDrawCommandList,
    MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
};
use crate::core::framework::render::{
    PrimitiveRelevance, RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents,
    ShaderQualityTier, packed_sort_key_u64,
};
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::resources::{MaterialDisabledPasses, PipelineKey};
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshBatchCacheIdentity {
    pub(crate) source_entity: EntityId,
    pub(crate) stable_instance_key: u64,
    pub(crate) draw_ordinal: u32,
}

#[derive(Clone)]
pub(crate) struct MeshBatchRef {
    pub(crate) source_draw_index: usize,
    pub(crate) cache_identity: Option<MeshBatchCacheIdentity>,
    pub(crate) queue_profile: MeshDrawQueueProfile,
    pub(crate) casts_shadow: bool,
    pub(crate) disabled_passes: MaterialDisabledPasses,
    pub(crate) primitive_relevance: Option<PrimitiveRelevance>,
    pub(crate) main_view_visible: bool,
    pub(crate) shadow_view_visible: bool,
    pub(crate) has_previous_velocity_transform: bool,
    pub(crate) taa_reactive_mask_strength: f32,
    pub(crate) half_resolution_transparency: bool,
    pub(crate) static_state: RenderMeshStaticState,
    pub(crate) pipeline_key: PipelineKey,
    pub(crate) sort_components: RenderPhaseSortComponents,
    pub(crate) geometry: MeshGeometryHandle,
    pub(crate) previous_velocity_geometry: Option<MeshGeometryHandle>,
    pub(crate) draw_args: MeshDrawArgs,
    pub(crate) gpu_scene_instance_span: Option<(u32, u32)>,
    pub(crate) gpu_scene_bind_group: Option<MeshBindHandle>,
    pub(crate) material_textures: Option<MeshBindHandle>,
    pub(crate) base_color_texture: Option<MeshBindHandle>,
    pub(crate) material: Option<MeshBindHandle>,
    pub(crate) standard_material: Option<MeshBindHandle>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MeshPassCommandSpec {
    pub(crate) phase: RenderPhase,
    pub(crate) pipeline_kind: MeshPassPipelineKind,
}

pub(crate) fn mesh_pass_command_specs(batch: &MeshBatchRef) -> [Option<MeshPassCommandSpec>; 6] {
    [
        depth_prepass_command_spec(batch),
        shadow_command_spec(batch),
        opaque_base_command_spec(batch),
        transparent_command_spec(batch),
        velocity_command_spec(batch),
        taa_reactive_mask_command_spec(batch),
    ]
}

pub(crate) fn depth_prepass_command_spec(batch: &MeshBatchRef) -> Option<MeshPassCommandSpec> {
    (!batch.disabled_passes.disables_depth_prepass()
        && batch.queue_profile.early_z_eligible()
        && batch.relevant_to_main_phase(RenderPhase::Prepass))
    .then_some(MeshPassCommandSpec {
        phase: RenderPhase::Prepass,
        pipeline_kind: MeshPassPipelineKind::DepthPrepass,
    })
}

pub(crate) fn shadow_command_spec(batch: &MeshBatchRef) -> Option<MeshPassCommandSpec> {
    if batch.disabled_passes.disables_shadow()
        || !batch.casts_shadow
        || !batch.relevant_to_shadow_view()
    {
        return None;
    }
    let pipeline_kind = match batch.phase() {
        MeshDrawQueuePhase::AlphaMask => MeshPassPipelineKind::ShadowDepthAlphaMask,
        MeshDrawQueuePhase::Opaque => MeshPassPipelineKind::ShadowDepth,
        MeshDrawQueuePhase::Transparent => return None,
    };
    Some(MeshPassCommandSpec {
        phase: RenderPhase::Shadow,
        pipeline_kind,
    })
}

pub(crate) fn opaque_base_command_spec(batch: &MeshBatchRef) -> Option<MeshPassCommandSpec> {
    if batch.disabled_passes.disables_base() {
        return None;
    }
    let phase = match batch.phase() {
        MeshDrawQueuePhase::Opaque => RenderPhase::Opaque3d,
        MeshDrawQueuePhase::AlphaMask => RenderPhase::AlphaMask3d,
        MeshDrawQueuePhase::Transparent => return None,
    };
    if !batch.relevant_to_main_phase(phase) {
        return None;
    }
    Some(MeshPassCommandSpec {
        phase: if batch.pipeline_key.requires_forward_path() {
            RenderPhase::Transparent3d
        } else {
            phase
        },
        pipeline_kind: MeshPassPipelineKind::Base,
    })
}

pub(crate) fn transparent_command_spec(batch: &MeshBatchRef) -> Option<MeshPassCommandSpec> {
    (!batch.disabled_passes.disables_base()
        && batch.phase() == MeshDrawQueuePhase::Transparent
        && batch.relevant_to_main_phase(RenderPhase::Transparent3d))
    .then_some(MeshPassCommandSpec {
        phase: RenderPhase::Transparent3d,
        pipeline_kind: MeshPassPipelineKind::Base,
    })
}

pub(crate) fn velocity_command_spec(batch: &MeshBatchRef) -> Option<MeshPassCommandSpec> {
    (!batch.disabled_passes.disables_velocity()
        && batch.queue_profile.early_z_eligible()
        && batch.queue_profile.velocity_history_eligible()
        && batch.has_previous_velocity_transform
        && batch.relevant_to_main_phase(RenderPhase::PostProcess))
    .then_some(MeshPassCommandSpec {
        phase: RenderPhase::PostProcess,
        pipeline_kind: MeshPassPipelineKind::Velocity,
    })
}

pub(crate) fn taa_reactive_mask_command_spec(batch: &MeshBatchRef) -> Option<MeshPassCommandSpec> {
    if batch.disabled_passes.disables_taa_reactive_mask() {
        return None;
    }
    let pipeline_kind = match batch.phase() {
        MeshDrawQueuePhase::Transparent
            if batch.relevant_to_main_phase(RenderPhase::Transparent3d) =>
        {
            MeshPassPipelineKind::TaaReactiveMask
        }
        MeshDrawQueuePhase::Opaque
            if batch.has_taa_reactive_material_mask()
                && batch.relevant_to_main_phase(RenderPhase::Opaque3d) =>
        {
            MeshPassPipelineKind::TaaReactiveMaterialMask
        }
        MeshDrawQueuePhase::AlphaMask
            if batch.has_taa_reactive_material_mask()
                && batch.relevant_to_main_phase(RenderPhase::AlphaMask3d) =>
        {
            MeshPassPipelineKind::TaaReactiveMaterialMask
        }
        _ => return None,
    };
    Some(MeshPassCommandSpec {
        phase: RenderPhase::PostProcess,
        pipeline_kind,
    })
}

impl MeshBatchRef {
    pub(crate) fn new(
        queue_profile: MeshDrawQueueProfile,
        pipeline_key: PipelineKey,
        sort_components: RenderPhaseSortComponents,
        geometry: MeshGeometryHandle,
        draw_args: MeshDrawArgs,
    ) -> Self {
        Self {
            source_draw_index: 0,
            cache_identity: None,
            queue_profile,
            casts_shadow: false,
            disabled_passes: MaterialDisabledPasses::default(),
            primitive_relevance: None,
            main_view_visible: true,
            shadow_view_visible: true,
            has_previous_velocity_transform: false,
            taa_reactive_mask_strength: 0.0,
            half_resolution_transparency: false,
            static_state: RenderMeshStaticState::default(),
            pipeline_key,
            sort_components,
            geometry,
            previous_velocity_geometry: None,
            draw_args,
            gpu_scene_instance_span: None,
            gpu_scene_bind_group: None,
            material_textures: None,
            base_color_texture: None,
            material: None,
            standard_material: None,
        }
    }

    pub(crate) fn with_source_draw_index(mut self, source_draw_index: usize) -> Self {
        self.source_draw_index = source_draw_index;
        self
    }

    #[cfg(test)]
    pub(crate) fn with_sort_components(
        mut self,
        sort_components: RenderPhaseSortComponents,
    ) -> Self {
        self.sort_components = sort_components;
        self
    }

    pub(crate) fn with_cache_identity(
        mut self,
        source_entity: EntityId,
        stable_instance_key: u64,
        draw_ordinal: u32,
    ) -> Self {
        self.cache_identity = Some(MeshBatchCacheIdentity {
            source_entity,
            stable_instance_key,
            draw_ordinal,
        });
        self
    }

    pub(crate) fn with_static_state(mut self, static_state: RenderMeshStaticState) -> Self {
        self.static_state = static_state;
        self
    }

    pub(crate) fn with_casts_shadow(mut self, casts_shadow: bool) -> Self {
        self.casts_shadow = casts_shadow;
        self
    }

    pub(crate) fn with_disabled_passes(mut self, disabled_passes: MaterialDisabledPasses) -> Self {
        self.disabled_passes = disabled_passes;
        self
    }

    pub(crate) fn with_visibility(
        mut self,
        primitive_relevance: Option<PrimitiveRelevance>,
        main_view_visible: bool,
        shadow_view_visible: bool,
    ) -> Self {
        self.primitive_relevance = primitive_relevance;
        self.main_view_visible = main_view_visible;
        self.shadow_view_visible = shadow_view_visible;
        self
    }

    pub(crate) fn with_previous_velocity_transform(mut self, present: bool) -> Self {
        self.has_previous_velocity_transform = present;
        self
    }

    pub(crate) fn with_previous_velocity_geometry(mut self, geometry: MeshGeometryHandle) -> Self {
        self.previous_velocity_geometry = Some(geometry);
        self
    }

    pub(crate) fn with_taa_reactive_mask_strength(mut self, strength: f32) -> Self {
        self.taa_reactive_mask_strength = if strength.is_finite() {
            strength.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self
    }

    pub(crate) fn with_half_resolution_transparency(mut self, enabled: bool) -> Self {
        self.half_resolution_transparency = enabled;
        self
    }

    pub(crate) fn has_taa_reactive_material_mask(&self) -> bool {
        self.taa_reactive_mask_strength > f32::EPSILON
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

    pub(crate) fn with_gpu_scene_bind_group(mut self, handle: MeshBindHandle) -> Self {
        self.gpu_scene_bind_group = Some(handle);
        self
    }

    pub(crate) fn with_material_textures(mut self, handle: MeshBindHandle) -> Self {
        self.material_textures = Some(handle);
        self
    }

    pub(crate) fn with_base_color_texture(mut self, handle: MeshBindHandle) -> Self {
        self.base_color_texture = Some(handle);
        self
    }

    pub(crate) fn with_material(mut self, handle: MeshBindHandle) -> Self {
        self.material = Some(handle);
        self
    }

    pub(crate) fn with_standard_material(mut self, handle: MeshBindHandle) -> Self {
        self.standard_material = Some(handle);
        self
    }

    pub(crate) fn phase(&self) -> MeshDrawQueuePhase {
        self.queue_profile.phase()
    }

    pub(crate) fn relevant_to_main_phase(&self, phase: RenderPhase) -> bool {
        if !self.main_view_visible {
            return false;
        }
        self.primitive_relevance
            .map(|relevance| relevance.is_relevant_to_phase(phase))
            .unwrap_or(true)
    }

    pub(crate) fn relevant_to_shadow_view(&self) -> bool {
        if !self.shadow_view_visible {
            return false;
        }
        self.primitive_relevance
            .map(PrimitiveRelevance::shadow_caster)
            .unwrap_or(self.casts_shadow)
    }

    pub(crate) fn command(
        &self,
        phase: RenderPhase,
        pipeline_kind: MeshPassPipelineKind,
        pipeline_variant_id: MeshPipelineVariantId,
    ) -> MeshDrawCommand {
        let sort_key = packed_sort_key_u64(
            phase,
            self.sort_components,
            pipeline_variant_id.value(),
            self.material_discriminant(),
        );
        let (first_instance_index, instance_count) = self
            .gpu_scene_instance_span
            .expect("mesh pass batches must carry a GPUScene instance span");
        let instance_source = DrawInstanceSource::GpuSceneInstance {
            first_instance_index,
            instance_count,
        };
        let draw_args = self
            .draw_args
            .clone()
            .with_instance_span(first_instance_index, instance_count);
        let mut command = MeshDrawCommand::new(
            phase,
            pipeline_kind,
            self.pipeline_key.clone(),
            pipeline_variant_id,
            sort_key,
            instance_source,
            self.geometry.clone(),
            draw_args,
        );
        if let Some(identity) = self.cache_identity {
            command = command.with_source_entity(identity.source_entity);
        }
        command = command.with_source_draw_index(self.source_draw_index);
        command = command.with_half_resolution_transparency(self.half_resolution_transparency);
        if let Some(material_textures) = &self.material_textures {
            command = command.with_material_textures(material_textures.clone());
        }
        if let Some(base_color_texture) = &self.base_color_texture {
            command = command.with_base_color_texture(base_color_texture.clone());
        }
        if let Some(material) = &self.material {
            command = command.with_material(material.clone());
        }
        if let Some(standard_material) = &self.standard_material {
            command = command.with_standard_material(standard_material.clone());
        }
        if let Some(gpu_scene_bind_group) = &self.gpu_scene_bind_group {
            command = command.with_gpu_scene_bind_group(gpu_scene_bind_group.clone());
        }
        if pipeline_kind == MeshPassPipelineKind::Velocity {
            if let Some(previous_velocity_geometry) = &self.previous_velocity_geometry {
                command =
                    command.with_previous_velocity_geometry(previous_velocity_geometry.clone());
            }
        }
        command
    }

    fn material_discriminant(&self) -> u16 {
        self.material
            .as_ref()
            .or(self.standard_material.as_ref())
            .map(|handle| handle.id() as u16)
            .unwrap_or_default()
    }
}

pub(crate) struct MeshPassBuildContext<'a, R: MeshPipelineVariantResolver + ?Sized> {
    variant_resolver: &'a mut R,
    shader_quality: ShaderQualityTier,
}

impl<'a, R> MeshPassBuildContext<'a, R>
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    pub(crate) fn new(variant_resolver: &'a mut R, shader_quality: ShaderQualityTier) -> Self {
        Self {
            variant_resolver,
            shader_quality,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_default_quality(variant_resolver: &'a mut R) -> Self {
        Self::new(variant_resolver, ShaderQualityTier::default())
    }

    pub(crate) fn pipeline_variant_id(
        &mut self,
        pipeline_kind: MeshPassPipelineKind,
        batch: &MeshBatchRef,
    ) -> MeshPipelineVariantId {
        self.variant_resolver.resolve_variant_for_geometry(
            pipeline_kind,
            &batch.pipeline_key,
            batch.queue_profile.shader_geometry_source_id(),
            self.shader_quality,
        )
    }
}

pub(crate) trait MeshPassProcessor {
    fn add_mesh_batch<R>(
        &mut self,
        batch: &MeshBatchRef,
        context: &mut MeshPassBuildContext<'_, R>,
        out: &mut MeshDrawCommandList,
    ) where
        R: MeshPipelineVariantResolver + ?Sized;
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GeometrySourceId,
        RenderPhase, RenderPhaseSortComponents, ShaderQualityTier,
    };
    use crate::core::framework::scene::Mobility;
    use crate::graphics::scene::resources::{PipelineKey, default_pipeline_key};
    use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
        MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

    use super::{
        MeshBatchRef, MeshDrawArgs, MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
    };

    #[test]
    fn mesh_batch_ref_attaches_previous_geometry_only_to_velocity_commands() {
        let batch = MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::DynamicGpuSkinningSource,
                Mobility::Dynamic,
                true,
                true,
                true,
            ),
            default_pipeline_key(),
            RenderPhaseSortComponents::new(0.0, 10),
            MeshGeometryHandle::test(1),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_gpu_scene_instance_span(0, 1)
        .with_previous_velocity_geometry(MeshGeometryHandle::test(2));

        let velocity = batch.command(
            RenderPhase::PostProcess,
            MeshPassPipelineKind::Velocity,
            MeshPipelineVariantId::new(1),
        );
        let base = batch.command(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Base,
            MeshPipelineVariantId::new(2),
        );

        assert_eq!(
            velocity
                .previous_velocity_geometry
                .as_ref()
                .map(MeshGeometryHandle::id),
            Some(2)
        );
        assert!(base.previous_velocity_geometry.is_none());
    }

    #[test]
    fn mesh_pass_build_context_resolves_prepared_gpu_skinning_as_skinned_variant() {
        let mut resolver = CapturingVariantResolver::default();
        let mut context =
            super::MeshPassBuildContext::new(&mut resolver, ShaderQualityTier::Medium);
        let batch = MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::Prepared,
                Mobility::Dynamic,
                false,
                true,
                false,
            ),
            default_pipeline_key(),
            RenderPhaseSortComponents::new(0.0, 10),
            MeshGeometryHandle::test(1),
            MeshDrawArgs::direct_indexed(0, 3),
        );

        let variant_id = context.pipeline_variant_id(MeshPassPipelineKind::Base, &batch);

        assert_eq!(variant_id, MeshPipelineVariantId::new(9));
        assert_eq!(
            resolver.last_geometry_source,
            Some(GEOMETRY_SOURCE_ID_SKINNED_MESH)
        );
    }

    #[test]
    fn mesh_pass_build_context_resolves_cpu_morphed_gpu_skinning_as_skinned_variant() {
        let mut resolver = CapturingVariantResolver::default();
        let mut context =
            super::MeshPassBuildContext::new(&mut resolver, ShaderQualityTier::Medium);
        let batch = MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::DynamicCpuMorphedGpuSkinningSource,
                Mobility::Dynamic,
                false,
                true,
                false,
            ),
            default_pipeline_key(),
            RenderPhaseSortComponents::new(0.0, 10),
            MeshGeometryHandle::test(1),
            MeshDrawArgs::direct_indexed(0, 3),
        );

        let variant_id = context.pipeline_variant_id(MeshPassPipelineKind::Base, &batch);

        assert_eq!(variant_id, MeshPipelineVariantId::new(9));
        assert_eq!(
            resolver.last_geometry_source,
            Some(GEOMETRY_SOURCE_ID_SKINNED_MESH)
        );
    }

    #[test]
    fn mesh_pass_build_context_resolves_gpu_skinned_morphed_as_skinned_morphed_variant() {
        let mut resolver = CapturingVariantResolver::default();
        let mut context =
            super::MeshPassBuildContext::new(&mut resolver, ShaderQualityTier::Medium);
        let batch = MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::DynamicGpuSkinnedMorphedSource,
                Mobility::Dynamic,
                false,
                true,
                false,
            ),
            default_pipeline_key(),
            RenderPhaseSortComponents::new(0.0, 10),
            MeshGeometryHandle::test(1),
            MeshDrawArgs::direct_indexed(0, 3),
        );

        let variant_id = context.pipeline_variant_id(MeshPassPipelineKind::Base, &batch);

        assert_eq!(variant_id, MeshPipelineVariantId::new(9));
        assert_eq!(
            resolver.last_geometry_source,
            Some(GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH)
        );
    }

    #[derive(Default)]
    struct CapturingVariantResolver {
        last_geometry_source: Option<GeometrySourceId>,
    }

    impl MeshPipelineVariantResolver for CapturingVariantResolver {
        fn resolve_variant_for_geometry(
            &mut self,
            _kind: MeshPassPipelineKind,
            _pipeline_key: &PipelineKey,
            geometry_source: GeometrySourceId,
            _shader_quality: ShaderQualityTier,
        ) -> MeshPipelineVariantId {
            self.last_geometry_source = Some(geometry_source);
            MeshPipelineVariantId::new(9)
        }
    }
}
