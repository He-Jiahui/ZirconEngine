use std::collections::HashMap;

use crate::core::framework::render::{RenderMeshStaticState, RenderPhase};
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawQueuePhase;

use super::{MeshBatchRef, MeshDrawCommand};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CachedMeshDrawKey {
    pub(crate) entity: EntityId,
    pub(crate) draw_ordinal: u32,
    pub(crate) phase: RenderPhase,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshDrawCommandCacheStats {
    pub(crate) cached_command_hit_count: usize,
    pub(crate) command_rebuild_count: usize,
    pub(crate) dynamic_command_count: usize,
}

#[derive(Default)]
pub(crate) struct CachedMeshDrawCommands {
    entries: HashMap<CachedMeshDrawKey, CachedMeshDrawEntry>,
}

struct CachedMeshDrawEntry {
    state: RenderMeshStaticState,
    command: MeshDrawCommand,
    last_touched_generation: u64,
}

impl CachedMeshDrawKey {
    pub(crate) fn from_batch_phase(batch: &MeshBatchRef, phase: RenderPhase) -> Option<Self> {
        let identity = batch.cache_identity?;
        Some(Self {
            entity: identity.entity,
            draw_ordinal: identity.draw_ordinal,
            phase,
        })
    }
}

impl CachedMeshDrawCommands {
    pub(crate) fn lookup(
        &mut self,
        key: &CachedMeshDrawKey,
        state: &RenderMeshStaticState,
        generation: u64,
    ) -> Option<MeshDrawCommand> {
        let entry = self.entries.get_mut(key)?;
        if entry.state != *state {
            return None;
        }
        entry.last_touched_generation = generation;
        Some(entry.command.clone())
    }

    pub(crate) fn store(
        &mut self,
        key: CachedMeshDrawKey,
        state: &RenderMeshStaticState,
        command: MeshDrawCommand,
        generation: u64,
    ) {
        self.entries.insert(
            key,
            CachedMeshDrawEntry {
                state: *state,
                command,
                last_touched_generation: generation,
            },
        );
    }

    pub(crate) fn retain_generation(&mut self, generation: u64) {
        self.entries
            .retain(|_, entry| entry.last_touched_generation == generation);
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
    use crate::core::framework::render::{RenderMeshStaticState, RenderPhase};
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
            entity: 7,
            draw_ordinal: 2,
            phase: RenderPhase::Opaque3d,
        };
        let state = RenderMeshStaticState::new(true, 11, 17);
        let command = test_command(RenderPhase::Opaque3d, 1);

        cache.store(key, &state, command.clone(), 3);

        let hit = cache
            .lookup(&key, &state, 4)
            .expect("matching state should hit");

        assert_eq!(hit.phase, command.phase);
        cache.retain_generation(4);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cached_mesh_draw_commands_invalidate_changed_material_revision() {
        let mut cache = CachedMeshDrawCommands::default();
        let key = CachedMeshDrawKey {
            entity: 7,
            draw_ordinal: 2,
            phase: RenderPhase::Opaque3d,
        };
        let state = RenderMeshStaticState::new(true, 11, 17);
        let changed_material = RenderMeshStaticState::new(true, 11, 23);

        cache.store(key, &state, test_command(RenderPhase::Opaque3d, 1), 1);

        assert!(cache.lookup(&key, &changed_material, 2).is_none());
        cache.retain_generation(2);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn cached_mesh_draw_commands_reject_dynamic_transparent_and_indirect_batches() {
        let static_state = RenderMeshStaticState::new(true, 11, 17);
        let opaque_static = batch(MeshDrawQueuePhase::Opaque, Mobility::Static, false)
            .with_cache_identity(7, 0)
            .with_static_state(static_state);
        let transparent_static = batch(MeshDrawQueuePhase::Transparent, Mobility::Static, false)
            .with_cache_identity(7, 0)
            .with_static_state(static_state);
        let indirect_static = batch(MeshDrawQueuePhase::Opaque, Mobility::Static, true)
            .with_cache_identity(7, 0)
            .with_static_state(static_state);
        let dynamic = batch(MeshDrawQueuePhase::Opaque, Mobility::Dynamic, false)
            .with_cache_identity(7, 0)
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
            1,
            MeshGeometryHandle::test(1),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_gpu_scene_instance_span(1, 1)
    }
}
