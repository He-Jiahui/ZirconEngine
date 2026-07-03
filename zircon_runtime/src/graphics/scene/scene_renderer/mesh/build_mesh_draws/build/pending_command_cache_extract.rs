use crate::core::framework::render::{RenderPhase, ShaderQualityTier};
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    CachedMeshDrawCommands, CachedMeshDrawKey, CachedMeshDrawLookup, MeshBatchRef, MeshDrawCommand,
    MeshDrawCommandCacheStats, MeshDrawCommandList, MeshPassBuildContext, MeshPassCommandBuffers,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::{
    MeshPipelineVariantRegistry, MeshPipelineVariantResolver,
};

use super::pending_command_cache_plan::PendingMeshCommandCacheVisibility;
use super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry};

mod extract_item;
#[cfg(test)]
mod fallback_tests;
#[cfg(test)]
mod lazy_rebuild_tests;
mod non_material_rebuild;
mod rebuild_batch;
mod residual_fallback;
#[cfg(test)]
mod second_frame_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod visibility_tests;

use extract_item::{
    cacheable_phases_for_extract_item, can_skip_pending_mesh_draw_for_cached_commands,
    pending_mesh_command_cache_extract_item, PendingMeshCommandCacheExtractItem,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PendingMeshCommandCacheExtractionStats {
    pub(crate) skipped_mesh_draw_count: usize,
    pub(crate) skipped_phase_count: usize,
    pub(crate) visibility_pruned_mesh_draw_count: usize,
    pub(crate) residual_material_phase_draw_count: usize,
    pub(crate) residual_rebuild_input_missing_draw_count: usize,
    pub(crate) residual_rebuild_rejected_draw_count: usize,
}

pub(crate) struct PendingMeshCommandCacheExtractionContext<'a> {
    command_cache: &'a mut CachedMeshDrawCommands,
    variant_resolver: &'a mut dyn MeshPipelineVariantResolver,
    generation: u64,
    shader_quality: ShaderQualityTier,
}

pub(super) struct PendingMeshCommandCacheExtraction {
    pub(super) pending_draws: Vec<Option<PendingMeshDraw>>,
    pub(super) command_buffers: MeshPassCommandBuffers,
    pub(super) stats: PendingMeshCommandCacheExtractionStats,
}

struct PendingMeshCommandCacheExtractedCommands {
    commands: Vec<MeshDrawCommand>,
    cache_stats: MeshDrawCommandCacheStats,
    visibility_pruned: bool,
}

impl<'a> PendingMeshCommandCacheExtractionContext<'a> {
    pub(crate) fn new(
        command_cache: &'a mut CachedMeshDrawCommands,
        variant_resolver: &'a mut dyn MeshPipelineVariantResolver,
        generation: u64,
        shader_quality: ShaderQualityTier,
    ) -> Self {
        Self {
            command_cache,
            variant_resolver,
            generation,
            shader_quality,
        }
    }
}

pub(super) fn extract_pending_static_mesh_command_cache_hits(
    mut pending_draws: Vec<Option<PendingMeshDraw>>,
    visibility_for_entity: impl Fn(EntityId) -> Option<PendingMeshCommandCacheVisibility>,
    gpu_scene_instance_span_for_draw: impl Fn(EntityId, u32) -> Option<(u32, u32)>,
    context: PendingMeshCommandCacheExtractionContext<'_>,
) -> PendingMeshCommandCacheExtraction {
    let mut commands = MeshDrawCommandList::new();
    let mut cache_stats = MeshDrawCommandCacheStats::default();
    let mut stats = PendingMeshCommandCacheExtractionStats::default();
    let command_cache = context.command_cache;
    let generation = context.generation;
    let mut build_context =
        MeshPassBuildContext::new(context.variant_resolver, context.shader_quality);

    for (source_draw_index, pending_draw) in pending_draws.iter_mut().enumerate() {
        let Some(draw) = pending_draw.as_ref() else {
            continue;
        };
        let item = pending_mesh_command_cache_extract_item(draw, source_draw_index);
        let visibility = visibility_for_entity(item.entity);
        let Some(extracted_commands) = commands_for_extract_item_with_stats_and_context(
            item,
            visibility,
            |phase| {
                rebuild_batch::pending_mesh_command_cache_rebuild_batch_for_phase(
                    draw,
                    item,
                    visibility,
                    phase,
                    &gpu_scene_instance_span_for_draw,
                )
            },
            &mut *command_cache,
            generation,
            &mut build_context,
            Some(&mut stats),
        ) else {
            continue;
        };
        stats.skipped_mesh_draw_count += 1;
        stats.skipped_phase_count += extracted_commands.commands.len();
        if extracted_commands.visibility_pruned {
            stats.visibility_pruned_mesh_draw_count += 1;
        }
        cache_stats.accumulate(extracted_commands.cache_stats);
        for command in extracted_commands.commands {
            commands.push(command);
        }
        *pending_draw = None;
    }

    PendingMeshCommandCacheExtraction {
        pending_draws,
        command_buffers: MeshPassCommandBuffers::from_cached_command_hits(commands, cache_stats),
        stats,
    }
}

fn cached_commands_for_extract_item(
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
) -> Option<Vec<MeshDrawCommand>> {
    commands_for_extract_item(item, visibility, |_| None, command_cache, generation)
        .map(|extracted_commands| extracted_commands.commands)
}

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
    commands_for_extract_item_with_stats_and_context(
        item,
        visibility,
        &mut rebuild_batch_for_phase,
        command_cache,
        generation,
        &mut build_context,
        extraction_stats,
    )
}

fn commands_for_extract_item_with_stats_and_context<R>(
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
    mut rebuild_batch_for_phase: impl FnMut(RenderPhase) -> Option<MeshBatchRef>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    build_context: &mut MeshPassBuildContext<'_, R>,
    mut extraction_stats: Option<&mut PendingMeshCommandCacheExtractionStats>,
) -> Option<PendingMeshCommandCacheExtractedCommands>
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    if !can_skip_pending_mesh_draw_for_cached_commands(item) {
        return None;
    }
    let phases = cacheable_phases_for_extract_item(item, visibility);
    if phases.is_empty() {
        return Some(PendingMeshCommandCacheExtractedCommands {
            commands: Vec::new(),
            cache_stats: MeshDrawCommandCacheStats::default(),
            visibility_pruned: true,
        });
    }

    let mut commands = Vec::with_capacity(phases.len());
    let mut rebuilt_commands = Vec::new();
    let mut cache_stats = MeshDrawCommandCacheStats::default();
    for phase in phases {
        let key = CachedMeshDrawKey {
            entity: item.entity,
            draw_ordinal: item.draw_ordinal,
            phase,
            disabled_passes: item.disabled_passes,
        };
        match command_cache.lookup_status(&key, &item.static_state, generation) {
            CachedMeshDrawLookup::Hit(command) => {
                cache_stats.cached_command_hit_count += 1;
                commands.push(
                    command
                        .with_source_entity(item.entity)
                        .with_source_draw_index(item.source_draw_index),
                );
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
                    return None;
                };
                cache_stats.cache_miss_count += 1;
                cache_stats.command_rebuild_count += 1;
                rebuilt_commands.push((key, command.clone()));
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
                    return None;
                };
                cache_stats.record_invalidation(invalidation);
                cache_stats.command_rebuild_count += 1;
                rebuilt_commands.push((key, command.clone()));
                commands.push(command);
            }
        };
    }

    for (key, command) in rebuilt_commands {
        command_cache.store(key, &item.static_state, command, generation);
    }

    Some(PendingMeshCommandCacheExtractedCommands {
        commands,
        cache_stats,
        visibility_pruned: false,
    })
}
