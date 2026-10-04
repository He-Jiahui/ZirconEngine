use std::collections::HashMap;
use std::sync::Arc;

use crate::core::framework::render::{
    RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents, ShaderQualityTier,
};
use crate::graphics::scene::resources::MaterialDisabledPasses;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawQueuePhase;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineResolverConfigurationEpoch;

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
    pub(crate) cache_invalidated_resolver_configuration_count: usize,
}

/// 跨帧复用静态命令 payload；命中后仍须投影本帧可见性、实例范围和变换，不能直接回放旧 command。
#[derive(Default)]
pub(crate) struct CachedMeshDrawCommands {
    entries: HashMap<CachedMeshDrawKey, CachedMeshDrawEntry>,
    pipeline_variant_pins: PipelineVariantPinCounts,
    resolver_configuration_epoch: MeshPipelineResolverConfigurationEpoch,
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
    pub(crate) fn synchronize_resolver_configuration(
        &mut self,
        epoch: MeshPipelineResolverConfigurationEpoch,
    ) -> usize {
        if self.resolver_configuration_epoch == epoch {
            return 0;
        }
        // resolver 配置改变会使 payload 的变体身份失效，命令与管线 pin 必须一起清除。
        self.resolver_configuration_epoch = epoch;
        let invalidated_count = self.entries.len();
        self.clear();
        invalidated_count
    }

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

    /// 本帧全部缓存抽取与残余命令构建完成后再清理；提前调用会丢弃尚未 touch 的命令及其变体 pin。
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
        self.cache_invalidated_resolver_configuration_count +=
            other.cache_invalidated_resolver_configuration_count;
    }

    pub(crate) fn record_resolver_configuration_invalidation(&mut self, count: usize) {
        self.cache_invalidated_resolver_configuration_count += count;
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
#[path = "tests/cached_mesh_draw_commands.rs"]
mod tests;
