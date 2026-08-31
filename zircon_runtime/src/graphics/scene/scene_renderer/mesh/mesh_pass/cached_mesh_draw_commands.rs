use std::collections::HashMap;
use std::sync::Arc;

use crate::core::framework::render::{
    RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents, ShaderQualityTier,
};
use crate::graphics::scene::resources::MaterialDisabledPasses;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawQueuePhase;

use super::pipeline_variant_pin_counts::PipelineVariantPinCounts;
use super::{MeshBatchRef, MeshDrawCommandPayload, MeshPipelineVariantId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CachedMeshDrawKey {
    pub(crate) stable_instance_key: u64,
    pub(crate) draw_ordinal: u32,
    pub(crate) phase: RenderPhase,
    pub(crate) disabled_passes: MaterialDisabledPasses,
    pub(crate) shader_quality: ShaderQualityTier,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshDrawCommandCacheStats {
    pub(crate) cached_command_hit_count: usize,
    pub(crate) command_rebuild_count: usize,
    pub(crate) dynamic_command_count: usize,
    pub(crate) cache_miss_count: usize,
    pub(crate) cache_invalidated_transform_count: usize,
    pub(crate) cache_invalidated_geometry_count: usize,
    pub(crate) cache_invalidated_material_count: usize,
}

#[derive(Default)]
pub(crate) struct CachedMeshDrawCommands {
    entries: HashMap<CachedMeshDrawKey, CachedMeshDrawEntry>,
    pipeline_variant_pins: PipelineVariantPinCounts,
}

struct CachedMeshDrawEntry {
    state: RenderMeshStaticState,
    payload: Arc<MeshDrawCommandPayload>,
    last_touched_generation: u64,
}

impl CachedMeshDrawKey {
    pub(crate) fn from_batch_phase(
        batch: &MeshBatchRef,
        phase: RenderPhase,
        shader_quality: ShaderQualityTier,
    ) -> Option<Self> {
        let identity = batch.cache_identity?;
        Some(Self {
            stable_instance_key: identity.stable_instance_key,
            draw_ordinal: identity.draw_ordinal,
            phase,
            disabled_passes: batch.disabled_passes,
            shader_quality,
        })
    }
}

impl CachedMeshDrawCommands {
    pub(crate) fn lookup_status(
        &mut self,
        key: &CachedMeshDrawKey,
        state: &RenderMeshStaticState,
        generation: u64,
    ) -> CachedMeshDrawLookup {
        let Some(entry) = self.entries.get_mut(key) else {
            return CachedMeshDrawLookup::Miss;
        };
        if entry.state != *state {
            return CachedMeshDrawLookup::Invalidated(CachedMeshDrawInvalidation::from_states(
                entry.state,
                *state,
            ));
        }
        entry.last_touched_generation = generation;
        CachedMeshDrawLookup::Hit(entry.payload.clone())
    }

    #[cfg(test)]
    pub(crate) fn lookup(
        &mut self,
        key: &CachedMeshDrawKey,
        state: &RenderMeshStaticState,
        generation: u64,
    ) -> Option<Arc<MeshDrawCommandPayload>> {
        match self.lookup_status(key, state, generation) {
            CachedMeshDrawLookup::Hit(payload) => Some(payload),
            CachedMeshDrawLookup::Miss | CachedMeshDrawLookup::Invalidated(_) => None,
        }
    }

    pub(crate) fn touch_if_state_matches(
        &mut self,
        key: &CachedMeshDrawKey,
        state: &RenderMeshStaticState,
        generation: u64,
    ) -> bool {
        let Some(entry) = self.entries.get_mut(key) else {
            return false;
        };
        if entry.state != *state {
            return false;
        }
        entry.last_touched_generation = generation;
        true
    }

    pub(crate) fn store(
        &mut self,
        key: CachedMeshDrawKey,
        state: &RenderMeshStaticState,
        payload: Arc<MeshDrawCommandPayload>,
        generation: u64,
    ) {
        assert!(
            payload.is_direct_indexed(),
            "cached mesh draw payloads must use direct indexed topology"
        );
        let variant_id = payload.pipeline_variant_id;
        let previous = self.entries.insert(
            key,
            CachedMeshDrawEntry {
                state: *state,
                payload,
                last_touched_generation: generation,
            },
        );
        match previous {
            Some(previous) => self
                .pipeline_variant_pins
                .replace(previous.payload.pipeline_variant_id, variant_id),
            None => self.pipeline_variant_pins.pin(variant_id),
        }
    }

    pub(crate) fn retain_generation(&mut self, generation: u64) {
        let pipeline_variant_pins = &mut self.pipeline_variant_pins;
        self.entries.retain(|_, entry| {
            let retain = entry.last_touched_generation == generation;
            if !retain {
                pipeline_variant_pins.unpin(entry.payload.pipeline_variant_id);
            }
            retain
        });
        crate::profile_counter!(
            "render",
            "mesh_pipeline_cpu_pinned_variant_count",
            self.pipeline_variant_pins.pinned_variant_count()
        );
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.pipeline_variant_pins.clear();
        crate::profile_counter!("render", "mesh_pipeline_cpu_pinned_variant_count", 0);
    }

    pub(crate) fn pins_pipeline_variant(&self, variant_id: MeshPipelineVariantId) -> bool {
        self.pipeline_variant_pins.is_pinned(variant_id)
    }

    pub(crate) fn is_cacheable_batch_phase(batch: &MeshBatchRef, phase: RenderPhase) -> bool {
        batch.cache_identity.is_some()
            && batch.static_state.has_authoritative_revisions()
            && batch.queue_profile.static_batch_eligible()
            && cacheable_phase_matches_batch(batch, phase)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

pub(crate) enum CachedMeshDrawLookup {
    Hit(Arc<MeshDrawCommandPayload>),
    Miss,
    Invalidated(CachedMeshDrawInvalidation),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CachedMeshDrawInvalidation {
    pub(crate) transform_changed: bool,
    pub(crate) geometry_changed: bool,
    pub(crate) material_changed: bool,
}

impl CachedMeshDrawInvalidation {
    const fn from_states(previous: RenderMeshStaticState, current: RenderMeshStaticState) -> Self {
        Self {
            transform_changed: previous.transform_static != current.transform_static,
            geometry_changed: previous.geometry_revision != current.geometry_revision,
            material_changed: previous.material_revision != current.material_revision,
        }
    }
}

impl MeshDrawCommandCacheStats {
    pub(crate) fn accumulate(&mut self, other: Self) {
        self.cached_command_hit_count += other.cached_command_hit_count;
        self.command_rebuild_count += other.command_rebuild_count;
        self.dynamic_command_count += other.dynamic_command_count;
        self.cache_miss_count += other.cache_miss_count;
        self.cache_invalidated_transform_count += other.cache_invalidated_transform_count;
        self.cache_invalidated_geometry_count += other.cache_invalidated_geometry_count;
        self.cache_invalidated_material_count += other.cache_invalidated_material_count;
    }

    pub(crate) fn record_invalidation(&mut self, invalidation: CachedMeshDrawInvalidation) {
        if invalidation.transform_changed {
            self.cache_invalidated_transform_count += 1;
        }
        if invalidation.geometry_changed {
            self.cache_invalidated_geometry_count += 1;
        }
        if invalidation.material_changed {
            self.cache_invalidated_material_count += 1;
        }
    }
}

fn cacheable_phase_matches_batch(batch: &MeshBatchRef, phase: RenderPhase) -> bool {
    match phase {
        RenderPhase::Prepass => batch.queue_profile.early_z_eligible(),
        RenderPhase::Shadow => batch.casts_shadow,
        RenderPhase::Opaque3d => batch.phase() == MeshDrawQueuePhase::Opaque,
        RenderPhase::AlphaMask3d => batch.phase() == MeshDrawQueuePhase::AlphaMask,
        RenderPhase::Transparent3d | RenderPhase::PostProcess => false,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents,
    };
    use crate::core::framework::scene::Mobility;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
        MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshBatchRef, MeshBindHandle, MeshDrawArgs, MeshDrawCommand,
        MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
    };

    use super::{CachedMeshDrawCommands, CachedMeshDrawKey};

    #[test]
    fn cached_mesh_draw_commands_reuse_matching_static_state() {
        let mut cache = CachedMeshDrawCommands::default();
        let key = CachedMeshDrawKey {
            stable_instance_key: (7 << 16) | 2,
            draw_ordinal: 2,
            phase: RenderPhase::Opaque3d,
            disabled_passes: Default::default(),
            shader_quality: Default::default(),
        };
        let state = RenderMeshStaticState::new(true, 11, 17);
        let command = test_command(RenderPhase::Opaque3d, 1);

        cache.store(key, &state, command.static_payload(), 3);
        assert!(cache.pins_pipeline_variant(command.pipeline_variant_id));

        let hit = cache
            .lookup(&key, &state, 4)
            .expect("matching state should hit");

        assert_eq!(hit.phase, command.phase);
        cache.retain_generation(4);
        assert_eq!(cache.len(), 1);
        assert!(cache.pins_pipeline_variant(command.pipeline_variant_id));
    }

    #[test]
    fn cached_mesh_draw_commands_clear_all_static_payloads() {
        let mut cache = CachedMeshDrawCommands::default();
        let key = CachedMeshDrawKey {
            stable_instance_key: (7 << 16) | 2,
            draw_ordinal: 2,
            phase: RenderPhase::Opaque3d,
            disabled_passes: Default::default(),
            shader_quality: Default::default(),
        };
        let state = RenderMeshStaticState::new(true, 11, 17);
        let command = test_command(RenderPhase::Opaque3d, 1);
        cache.store(key, &state, command.static_payload(), 3);
        assert!(cache.pins_pipeline_variant(command.pipeline_variant_id));

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(!cache.pins_pipeline_variant(command.pipeline_variant_id));
        assert!(cache.lookup(&key, &state, 4).is_none());
    }

    #[test]
    fn cached_mesh_draw_commands_invalidate_changed_material_revision() {
        let mut cache = CachedMeshDrawCommands::default();
        let key = CachedMeshDrawKey {
            stable_instance_key: (7 << 16) | 2,
            draw_ordinal: 2,
            phase: RenderPhase::Opaque3d,
            disabled_passes: Default::default(),
            shader_quality: Default::default(),
        };
        let state = RenderMeshStaticState::new(true, 11, 17);
        let changed_material = RenderMeshStaticState::new(true, 11, 23);

        cache.store(
            key,
            &state,
            test_command(RenderPhase::Opaque3d, 1).static_payload(),
            1,
        );

        assert!(cache.lookup(&key, &changed_material, 2).is_none());
        cache.retain_generation(2);
        assert_eq!(cache.len(), 0);
        assert!(!cache.pins_pipeline_variant(MeshPipelineVariantId::new(1)));
    }

    #[test]
    fn cached_mesh_draw_commands_reject_dynamic_transparent_and_indirect_batches() {
        let static_state = RenderMeshStaticState::new(true, 11, 17);
        let opaque_static = batch(MeshDrawQueuePhase::Opaque, Mobility::Static, false)
            .with_cache_identity(7, 7 << 16, 0)
            .with_static_state(static_state);
        let transparent_static = batch(MeshDrawQueuePhase::Transparent, Mobility::Static, false)
            .with_cache_identity(7, 7 << 16, 0)
            .with_static_state(static_state);
        let indirect_static = batch(MeshDrawQueuePhase::Opaque, Mobility::Static, true)
            .with_cache_identity(7, 7 << 16, 0)
            .with_static_state(static_state);
        let dynamic = batch(MeshDrawQueuePhase::Opaque, Mobility::Dynamic, false)
            .with_cache_identity(7, 7 << 16, 0)
            .with_static_state(static_state);

        assert!(CachedMeshDrawCommands::is_cacheable_batch_phase(
            &opaque_static,
            RenderPhase::Opaque3d
        ));
        assert!(!CachedMeshDrawCommands::is_cacheable_batch_phase(
            &transparent_static,
            RenderPhase::Transparent3d
        ));
        assert!(!CachedMeshDrawCommands::is_cacheable_batch_phase(
            &indirect_static,
            RenderPhase::Opaque3d
        ));
        assert!(!CachedMeshDrawCommands::is_cacheable_batch_phase(
            &dynamic,
            RenderPhase::Opaque3d
        ));
    }

    #[test]
    #[should_panic(expected = "cached mesh draw payloads must use direct indexed topology")]
    fn cached_mesh_draw_commands_reject_indirect_payload_storage() {
        let mut cache = CachedMeshDrawCommands::default();
        let key = CachedMeshDrawKey {
            stable_instance_key: 7 << 16,
            draw_ordinal: 0,
            phase: RenderPhase::Opaque3d,
            disabled_passes: Default::default(),
            shader_quality: Default::default(),
        };
        let state = RenderMeshStaticState::new(true, 11, 17);
        let command = MeshDrawCommand::new(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            1,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: 1,
                instance_count: 1,
            },
            MeshGeometryHandle::test(1),
            MeshDrawArgs::test_indexed_indirect(91, 0),
        );

        cache.store(key, &state, command.static_payload(), 1);
    }

    #[test]
    fn cached_mesh_draw_commands_keep_sibling_primitives_separate() {
        let mut cache = CachedMeshDrawCommands::default();
        let state = RenderMeshStaticState::new(true, 11, 17);
        let first = CachedMeshDrawKey {
            stable_instance_key: 7 << 16,
            draw_ordinal: 0,
            phase: RenderPhase::Opaque3d,
            disabled_passes: Default::default(),
            shader_quality: Default::default(),
        };
        let second = CachedMeshDrawKey {
            stable_instance_key: (7 << 16) | 1,
            ..first
        };

        cache.store(
            first,
            &state,
            test_command(RenderPhase::Opaque3d, 1).static_payload(),
            1,
        );
        cache.store(
            second,
            &state,
            test_command(RenderPhase::Opaque3d, 2).static_payload(),
            1,
        );

        assert_eq!(cache.len(), 2);
        assert!(cache.lookup(&first, &state, 2).is_some());
        assert!(cache.lookup(&second, &state, 2).is_some());
    }

    fn test_command(phase: RenderPhase, sort_key: u64) -> MeshDrawCommand {
        MeshDrawCommand::new(
            phase,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            sort_key,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: 1,
                instance_count: 1,
            },
            MeshGeometryHandle::test(1),
            MeshDrawArgs::direct_indexed(0, 3).with_instance_span(1, 1),
        )
    }

    fn batch(phase: MeshDrawQueuePhase, mobility: Mobility, indirect: bool) -> MeshBatchRef {
        MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                phase,
                MeshDrawGeometrySource::Prepared,
                mobility,
                indirect,
                false,
                false,
            ),
            default_pipeline_key(),
            RenderPhaseSortComponents::new(0.0, 1),
            MeshGeometryHandle::test(1),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_gpu_scene_instance_span(1, 1)
    }
}
