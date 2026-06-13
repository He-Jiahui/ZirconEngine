use super::{
    DrawInstanceSource, MeshBindHandle, MeshDrawArgs, MeshDrawCommand, MeshDrawCommandList,
    MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
};
use crate::core::framework::render::{
    packed_sort_key_u64, PrimitiveRelevance, RenderMeshStaticState, RenderPhase,
    RenderPhaseSortComponents,
};
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshBatchCacheIdentity {
    pub(crate) entity: EntityId,
    pub(crate) draw_ordinal: u32,
}

#[derive(Clone)]
pub(crate) struct MeshBatchRef {
    pub(crate) source_draw_index: usize,
    pub(crate) cache_identity: Option<MeshBatchCacheIdentity>,
    pub(crate) queue_profile: MeshDrawQueueProfile,
    pub(crate) casts_shadow: bool,
    pub(crate) primitive_relevance: Option<PrimitiveRelevance>,
    pub(crate) main_view_visible: bool,
    pub(crate) shadow_view_visible: bool,
    pub(crate) has_previous_motion_vector_transform: bool,
    pub(crate) static_state: RenderMeshStaticState,
    pub(crate) pipeline_key: PipelineKey,
    pub(crate) sort_key: u64,
    pub(crate) geometry: MeshGeometryHandle,
    pub(crate) draw_args: MeshDrawArgs,
    pub(crate) gpu_scene_instance_span: Option<(u32, u32)>,
    pub(crate) gpu_scene_bind_group: Option<MeshBindHandle>,
    pub(crate) material_textures: Option<MeshBindHandle>,
    pub(crate) base_color_texture: Option<MeshBindHandle>,
    pub(crate) material: Option<MeshBindHandle>,
    pub(crate) standard_material: Option<MeshBindHandle>,
}

impl MeshBatchRef {
    pub(crate) fn new(
        queue_profile: MeshDrawQueueProfile,
        pipeline_key: PipelineKey,
        sort_key: u64,
        geometry: MeshGeometryHandle,
        draw_args: MeshDrawArgs,
    ) -> Self {
        Self {
            source_draw_index: 0,
            cache_identity: None,
            queue_profile,
            casts_shadow: false,
            primitive_relevance: None,
            main_view_visible: true,
            shadow_view_visible: true,
            has_previous_motion_vector_transform: false,
            static_state: RenderMeshStaticState::default(),
            pipeline_key,
            sort_key,
            geometry,
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

    pub(crate) fn with_cache_identity(mut self, entity: EntityId, draw_ordinal: u32) -> Self {
        self.cache_identity = Some(MeshBatchCacheIdentity {
            entity,
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

    pub(crate) fn with_previous_motion_vector_transform(mut self, present: bool) -> Self {
        self.has_previous_motion_vector_transform = present;
        self
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
            RenderPhaseSortComponents::new(0.0, self.sort_key),
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
            command = command.with_source_entity(identity.entity);
        }
        command = command.with_source_draw_index(self.source_draw_index);
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
}

impl<'a, R> MeshPassBuildContext<'a, R>
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    pub(crate) fn new(variant_resolver: &'a mut R) -> Self {
        Self { variant_resolver }
    }

    pub(crate) fn pipeline_variant_id(
        &mut self,
        pipeline_kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
    ) -> MeshPipelineVariantId {
        self.variant_resolver
            .resolve_variant(pipeline_kind, pipeline_key)
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
