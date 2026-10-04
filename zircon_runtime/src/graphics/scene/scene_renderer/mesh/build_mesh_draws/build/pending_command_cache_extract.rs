use crate::core::framework::render::{RenderPhase, ShaderQualityTier};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    CachedMeshDrawCommands, CachedMeshDrawKey, CachedMeshDrawLookup, MeshBatchRef, MeshDrawCommand,
    MeshDrawCommandCacheStats, MeshDrawCommandList, MeshPassBuildContext, MeshPassCommandBuffers,
};
#[cfg(test)]
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantRegistry;
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantResolver;

use super::pending_command_cache_plan::PendingMeshCommandCacheVisibility;
use super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry};

mod command_slots;
mod extract_item;
#[cfg(test)]
#[path = "pending_command_cache_extract/tests/fallback_tests.rs"]
mod fallback_tests;
#[cfg(test)]
#[path = "pending_command_cache_extract/tests/lazy_rebuild_tests.rs"]
mod lazy_rebuild_tests;
mod non_material_rebuild;
mod rebuild_batch;
mod remainder;
mod residual_fallback;
#[cfg(test)]
#[path = "pending_command_cache_extract/tests/second_frame_tests.rs"]
mod second_frame_tests;
#[cfg(test)]
#[path = "pending_command_cache_extract/tests/cases.rs"]
mod tests;
#[cfg(test)]
#[path = "pending_command_cache_extract/tests/visibility_tests.rs"]
mod visibility_tests;

use command_slots::PendingMeshCommandSlots;
use extract_item::{
    cacheable_phase_slots_for_extract_item, cacheable_phases_for_extract_item,
    can_skip_pending_mesh_draw_for_cached_commands, pending_mesh_command_cache_extract_item,
    PendingMeshCommandCacheExtractItem,
};
pub(super) use remainder::PendingMeshDrawRemainder;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PendingMeshCommandCacheExtractionStats {
    pub(crate) skipped_mesh_draw_count: usize,
    pub(crate) skipped_phase_count: usize,
    pub(crate) visibility_pruned_mesh_draw_count: usize,
    pub(crate) residual_material_phase_draw_count: usize,
    pub(crate) residual_rebuild_input_missing_draw_count: usize,
    pub(crate) residual_rebuild_rejected_draw_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PendingMeshCommandMaterializationAttemptStats {
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    discarded_command_count: usize,
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    discarded_cache_hit_payload_arc_clone_count: usize,
}

impl PendingMeshCommandMaterializationAttemptStats {
    #[inline]
    fn record_discarded(&mut self, command_count: usize, cache_hit_count: usize) {
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        {
            self.discarded_command_count += command_count;
            self.discarded_cache_hit_payload_arc_clone_count += cache_hit_count;
        }
        #[cfg(not(any(test, feature = "profiling", feature = "profiling-tracy")))]
        let _ = (command_count, cache_hit_count);
    }
}

pub(crate) struct PendingMeshCommandCacheExtractionContext<'a> {
    command_cache: &'a mut CachedMeshDrawCommands,
    generation: u64,
    shader_quality: ShaderQualityTier,
}

pub(super) struct PendingMeshCommandCacheExtraction {
    pub(super) pending_draws: PendingMeshDrawRemainder,
    pub(super) command_buffers: MeshPassCommandBuffers,
    pub(super) stats: PendingMeshCommandCacheExtractionStats,
}

struct PendingMeshCommandCacheExtractedCommands {
    commands: PendingMeshCommandSlots<MeshDrawCommand>,
    cache_stats: MeshDrawCommandCacheStats,
    visibility_pruned: bool,
}

impl<'a> PendingMeshCommandCacheExtractionContext<'a> {
    pub(crate) fn new(
        command_cache: &'a mut CachedMeshDrawCommands,
        generation: u64,
        shader_quality: ShaderQualityTier,
    ) -> Self {
        Self {
            command_cache,
            generation,
            shader_quality,
        }
    }
}

/// 仅当所需 phase 均可复用或重建时移除整项；中途无法完成的项保留原 source index，交回普通 MeshDraw 构建。
pub(super) fn extract_pending_static_mesh_command_cache_hits(
    pending_draws: Vec<PendingMeshDraw>,
    visibility_for_instance: impl Fn(u64) -> Option<PendingMeshCommandCacheVisibility>,
    gpu_scene_instance_span_for_instance: impl Fn(u64) -> Option<(u32, u32)>,
    variant_resolver: &mut dyn MeshPipelineVariantResolver,
    context: PendingMeshCommandCacheExtractionContext<'_>,
) -> PendingMeshCommandCacheExtraction {
    let (commands, cache_stats, stats, materialization_attempt_stats, residual_pending_draws) = {
        crate::profile_scope!("render", "mesh_commands", "extract_cached_pre_mesh_draw");
        let mut commands = MeshDrawCommandList::new();
        let mut cache_stats = MeshDrawCommandCacheStats::default();
        let mut stats = PendingMeshCommandCacheExtractionStats::default();
        let mut materialization_attempt_stats =
            PendingMeshCommandMaterializationAttemptStats::default();
        let command_cache = context.command_cache;
        let generation = context.generation;
        cache_stats.record_resolver_configuration_invalidation(
            command_cache
                .synchronize_resolver_configuration(variant_resolver.configuration_epoch()),
        );
        let mut build_context = MeshPassBuildContext::new(variant_resolver, context.shader_quality);
        let residual_capacity = pending_draws.len();
        let mut residual_pending_draws = None;

        for (source_draw_index, draw) in pending_draws.into_iter().enumerate() {
            let mut item = pending_mesh_command_cache_extract_item(&draw, source_draw_index);
            if can_skip_pending_mesh_draw_for_cached_commands(item) {
                // Extraction visits every pending draw. Only static cache candidates need their
                // current span; hit and miss handling below reuse this one synchronized lookup.
                item.gpu_scene_instance_span =
                    gpu_scene_instance_span_for_instance(item.stable_instance_key);
            }
            let visibility = visibility_for_instance(item.stable_instance_key);
            let Some(extracted_commands) = commands_for_extract_item_with_stats_and_context(
                item,
                visibility,
                |phase| {
                    rebuild_batch::pending_mesh_command_cache_rebuild_batch_for_phase(
                        &draw, item, visibility, phase,
                    )
                },
                &mut *command_cache,
                generation,
                &mut build_context,
                Some(&mut stats),
                &mut materialization_attempt_stats,
            ) else {
                residual_pending_draws
                    .get_or_insert_with(|| Vec::with_capacity(residual_capacity))
                    .push((source_draw_index, draw));
                continue;
            };
            stats.skipped_mesh_draw_count += 1;
            stats.skipped_phase_count += extracted_commands.commands.len();
            if extracted_commands.visibility_pruned {
                stats.visibility_pruned_mesh_draw_count += 1;
            }
            cache_stats.accumulate(extracted_commands.cache_stats);
            #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
            commands.record_staging_merge(
                extracted_commands.commands.len(),
                extracted_commands.commands.len(),
            );
            for command in extracted_commands.commands {
                commands.push(command);
            }
        }

        (
            commands,
            cache_stats,
            stats,
            materialization_attempt_stats,
            residual_pending_draws,
        )
    };

    let command_buffers = {
        crate::profile_scope!("render", "mesh_commands", "pre_mesh_materialize");
        MeshPassCommandBuffers::from_cached_command_hits(commands, cache_stats)
    };
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    let mut command_buffers = command_buffers;
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    command_buffers.record_discarded_materialization_attempt(
        materialization_attempt_stats.discarded_command_count,
        materialization_attempt_stats.discarded_cache_hit_payload_arc_clone_count,
    );
    #[cfg(not(any(test, feature = "profiling", feature = "profiling-tracy")))]
    let _ = materialization_attempt_stats;

    PendingMeshCommandCacheExtraction {
        pending_draws: PendingMeshDrawRemainder::Residual(
            residual_pending_draws.unwrap_or_default(),
        ),
        command_buffers,
        stats,
    }
}

#[cfg(test)]
fn cached_commands_for_extract_item(
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
) -> Option<Vec<MeshDrawCommand>> {
    commands_for_extract_item(item, visibility, |_| None, command_cache, generation)
        .map(|extracted_commands| extracted_commands.commands.into_iter().collect())
}

#[cfg(test)]
fn commands_for_extract_item(
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
    mut rebuild_batch_for_phase: impl FnMut(RenderPhase) -> Option<MeshBatchRef>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
) -> Option<PendingMeshCommandCacheExtractedCommands> {
    commands_for_extract_item_with_stats(
        item,
        visibility,
        &mut rebuild_batch_for_phase,
        command_cache,
        generation,
        None,
    )
}

#[cfg(test)]
fn commands_for_extract_item_with_stats(
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
    mut rebuild_batch_for_phase: impl FnMut(RenderPhase) -> Option<MeshBatchRef>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    extraction_stats: Option<&mut PendingMeshCommandCacheExtractionStats>,
) -> Option<PendingMeshCommandCacheExtractedCommands> {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut build_context = MeshPassBuildContext::new(&mut variants, ShaderQualityTier::default());
    let mut materialization_attempt_stats =
        PendingMeshCommandMaterializationAttemptStats::default();
    commands_for_extract_item_with_stats_and_context(
        item,
        visibility,
        &mut rebuild_batch_for_phase,
        command_cache,
        generation,
        &mut build_context,
        extraction_stats,
        &mut materialization_attempt_stats,
    )
}

#[cfg(test)]
fn commands_for_extract_item_with_materialization_stats(
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
    mut rebuild_batch_for_phase: impl FnMut(RenderPhase) -> Option<MeshBatchRef>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
) -> (
    Option<PendingMeshCommandCacheExtractedCommands>,
    PendingMeshCommandMaterializationAttemptStats,
) {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut build_context = MeshPassBuildContext::new(&mut variants, ShaderQualityTier::default());
    let mut materialization_attempt_stats =
        PendingMeshCommandMaterializationAttemptStats::default();
    let extracted = commands_for_extract_item_with_stats_and_context(
        item,
        visibility,
        &mut rebuild_batch_for_phase,
        command_cache,
        generation,
        &mut build_context,
        None,
        &mut materialization_attempt_stats,
    );
    (extracted, materialization_attempt_stats)
}

fn commands_for_extract_item_with_stats_and_context<R>(
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
    mut rebuild_batch_for_phase: impl FnMut(RenderPhase) -> Option<MeshBatchRef>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    build_context: &mut MeshPassBuildContext<'_, R>,
    mut extraction_stats: Option<&mut PendingMeshCommandCacheExtractionStats>,
    materialization_attempt_stats: &mut PendingMeshCommandMaterializationAttemptStats,
) -> Option<PendingMeshCommandCacheExtractedCommands>
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    if !can_skip_pending_mesh_draw_for_cached_commands(item)
        || item.gpu_scene_instance_span.is_none()
    {
        return None;
    }
    let phases = cacheable_phases_for_extract_item(item, visibility);
    for phase in cacheable_phase_slots_for_extract_item(item)
        .into_iter()
        .flatten()
    {
        if phases.contains(&Some(phase)) {
            continue;
        }
        let key = CachedMeshDrawKey {
            stable_instance_key: item.stable_instance_key,
            draw_ordinal: item.draw_ordinal,
            phase,
            disabled_passes: item.disabled_passes,
            shader_quality: build_context.shader_quality(),
        };
        // View selection does not own cache residency. A matching static source stays live even
        // when this view omits the command, while state mismatches retire at frame end.
        command_cache.touch_if_state_matches(&key, &item.static_state, generation);
    }
    if phases.iter().all(Option::is_none) {
        return Some(PendingMeshCommandCacheExtractedCommands {
            commands: PendingMeshCommandSlots::default(),
            cache_stats: MeshDrawCommandCacheStats::default(),
            visibility_pruned: true,
        });
    }

    let mut commands = PendingMeshCommandSlots::default();
    let mut rebuilt_commands = PendingMeshCommandSlots::default();
    let mut cache_stats = MeshDrawCommandCacheStats::default();
    for phase in phases.into_iter().flatten() {
        let key = CachedMeshDrawKey {
            stable_instance_key: item.stable_instance_key,
            draw_ordinal: item.draw_ordinal,
            phase,
            disabled_passes: item.disabled_passes,
            shader_quality: build_context.shader_quality(),
        };
        match command_cache.lookup_status(&key, &item.static_state, generation) {
            CachedMeshDrawLookup::Hit(payload) => {
                cache_stats.cached_command_hit_count += 1;
                let gpu_scene_instance_span = item
                    .gpu_scene_instance_span
                    .expect("cacheable pending mesh draws must have a synchronized GPUScene span");
                commands.push(MeshDrawCommand::from_cached_payload(
                    payload,
                    item.entity,
                    item.source_draw_index,
                    item.sort_components,
                    gpu_scene_instance_span,
                    None,
                ));
            }
            CachedMeshDrawLookup::Miss => {
                let Some(command) =
                    residual_fallback::rebuild_non_material_command_or_record_residual(
                        &mut rebuild_batch_for_phase,
                        phase,
                        build_context,
                        &mut extraction_stats,
                    )
                else {
                    materialization_attempt_stats
                        .record_discarded(commands.len(), cache_stats.cached_command_hit_count);
                    return None;
                };
                cache_stats.cache_miss_count += 1;
                cache_stats.command_rebuild_count += 1;
                let (command, payload) = command.into_shared_payload();
                rebuilt_commands.push((key, payload));
                commands.push(command);
            }
            CachedMeshDrawLookup::Invalidated(invalidation) => {
                let Some(command) =
                    residual_fallback::rebuild_non_material_command_or_record_residual(
                        &mut rebuild_batch_for_phase,
                        phase,
                        build_context,
                        &mut extraction_stats,
                    )
                else {
                    materialization_attempt_stats
                        .record_discarded(commands.len(), cache_stats.cached_command_hit_count);
                    return None;
                };
                cache_stats.record_invalidation(invalidation);
                cache_stats.command_rebuild_count += 1;
                let (command, payload) = command.into_shared_payload();
                rebuilt_commands.push((key, payload));
                commands.push(command);
            }
        };
    }

    // 所有可见 phase 都已生成后才写回重建 payload；中途退出不会把半套新命令存进缓存。
    for (key, command) in rebuilt_commands {
        command_cache.store(key, &item.static_state, command, generation);
    }

    Some(PendingMeshCommandCacheExtractedCommands {
        commands,
        cache_stats,
        visibility_pruned: false,
    })
}
