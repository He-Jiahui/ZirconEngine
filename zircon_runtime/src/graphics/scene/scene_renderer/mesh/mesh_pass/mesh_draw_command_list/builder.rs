use crate::core::framework::render::{RenderPhase, ShaderQualityTier};

use super::super::super::MeshDraw;
use super::super::super::mesh_draw::MeshDrawQueuePhase;
use super::super::super::mesh_pipeline_cache::MeshPipelineVariantResolver;
use super::super::cached_mesh_draw_commands::{
    CachedMeshDrawCommands, CachedMeshDrawKey, CachedMeshDrawLookup, MeshDrawCommandCacheStats,
};
use super::super::mesh_pass_processor::{MeshBatchRef, MeshPassBuildContext, MeshPassProcessor};
use super::super::processors::{
    DepthPrepassProcessor, OpaqueBasePassProcessor, ShadowPassProcessor,
    TaaReactiveMaskPassProcessor, TransparentPassProcessor, VelocityPassProcessor,
};
use super::{MeshDrawCommandList, MeshPassCommandBuffers};

pub(crate) fn build_mesh_pass_command_buffers<R>(
    draws: &[MeshDraw],
    variant_resolver: &mut R,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    build_mesh_pass_command_buffers_from_batches(
        draws
            .iter()
            .enumerate()
            .map(|(draw_index, draw)| draw.mesh_pass_batch_ref(draw_index as u64, draw_index)),
        variant_resolver,
        ShaderQualityTier::default(),
    )
}

pub(crate) fn build_mesh_pass_command_buffers_cached<R>(
    draws: &[MeshDraw],
    variant_resolver: &mut R,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    shader_quality: ShaderQualityTier,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    build_mesh_pass_command_buffers_from_batches_cached(
        draws
            .iter()
            .enumerate()
            .map(|(draw_index, draw)| draw.mesh_pass_batch_ref(draw_index as u64, draw_index)),
        variant_resolver,
        command_cache,
        generation,
        shader_quality,
    )
}

pub(super) fn build_mesh_pass_command_buffers_from_batches<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
    shader_quality: ShaderQualityTier,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    build_mesh_pass_command_buffers_from_batches_uncached(batches, variant_resolver, shader_quality)
}

fn build_mesh_pass_command_buffers_from_batches_uncached<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
    shader_quality: ShaderQualityTier,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let mut depth_prepass = DepthPrepassProcessor;
    let mut opaque_base = OpaqueBasePassProcessor;
    let mut transparent = TransparentPassProcessor;
    let mut shadow = ShadowPassProcessor;
    let mut velocity = VelocityPassProcessor;
    let mut taa_reactive_mask = TaaReactiveMaskPassProcessor;
    let mut commands = MeshDrawCommandList::new();
    let mut build_context = MeshPassBuildContext::new(variant_resolver, shader_quality);
    let mut cache_stats = MeshDrawCommandCacheStats::default();

    for batch in batches {
        let command_start = commands.commands().len();
        depth_prepass.add_mesh_batch(&batch, &mut build_context, &mut commands);
        shadow.add_mesh_batch(&batch, &mut build_context, &mut commands);
        opaque_base.add_mesh_batch(&batch, &mut build_context, &mut commands);
        transparent.add_mesh_batch(&batch, &mut build_context, &mut commands);
        velocity.add_mesh_batch(&batch, &mut build_context, &mut commands);
        taa_reactive_mask.add_mesh_batch(&batch, &mut build_context, &mut commands);
        record_dynamic_commands_since(&commands, command_start, &mut cache_stats);
    }

    MeshPassCommandBuffers::from_command_list(commands, cache_stats)
}

pub(super) fn build_mesh_pass_command_buffers_from_batches_cached<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    shader_quality: ShaderQualityTier,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let mut depth_prepass = DepthPrepassProcessor;
    let mut opaque_base = OpaqueBasePassProcessor;
    let mut transparent = TransparentPassProcessor;
    let mut shadow = ShadowPassProcessor;
    let mut velocity = VelocityPassProcessor;
    let mut taa_reactive_mask = TaaReactiveMaskPassProcessor;
    let mut commands = MeshDrawCommandList::new();
    let mut build_context = MeshPassBuildContext::new(variant_resolver, shader_quality);
    let mut cache_stats = MeshDrawCommandCacheStats::default();

    for batch in batches {
        if add_cached_static_batch(
            &batch,
            &mut build_context,
            command_cache,
            generation,
            &mut commands,
            &mut cache_stats,
        ) {
            continue;
        }

        let command_start = commands.commands().len();
        depth_prepass.add_mesh_batch(&batch, &mut build_context, &mut commands);
        shadow.add_mesh_batch(&batch, &mut build_context, &mut commands);
        opaque_base.add_mesh_batch(&batch, &mut build_context, &mut commands);
        transparent.add_mesh_batch(&batch, &mut build_context, &mut commands);
        velocity.add_mesh_batch(&batch, &mut build_context, &mut commands);
        taa_reactive_mask.add_mesh_batch(&batch, &mut build_context, &mut commands);
        record_dynamic_commands_since(&commands, command_start, &mut cache_stats);
    }

    MeshPassCommandBuffers::from_command_list(commands, cache_stats)
}

fn add_cached_static_batch<R>(
    batch: &MeshBatchRef,
    build_context: &mut MeshPassBuildContext<'_, R>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    commands: &mut MeshDrawCommandList,
    cache_stats: &mut MeshDrawCommandCacheStats,
) -> bool
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    if !batch.static_state.has_authoritative_revisions()
        || !batch.queue_profile.static_batch_eligible()
        || batch.cache_identity.is_none()
        || batch.pipeline_key.requires_forward_path()
    {
        return false;
    }

    if batch.queue_profile.early_z_eligible() && batch.relevant_to_main_phase(RenderPhase::Prepass)
    {
        add_cached_or_rebuilt_phase(
            batch,
            RenderPhase::Prepass,
            build_context,
            command_cache,
            generation,
            commands,
            cache_stats,
            |batch, context, out| DepthPrepassProcessor.add_mesh_batch(batch, context, out),
        );
    }
    if batch.casts_shadow && batch.relevant_to_shadow_view() {
        add_cached_or_rebuilt_phase(
            batch,
            RenderPhase::Shadow,
            build_context,
            command_cache,
            generation,
            commands,
            cache_stats,
            |batch, context, out| ShadowPassProcessor.add_mesh_batch(batch, context, out),
        );
    }
    match batch.phase() {
        MeshDrawQueuePhase::Opaque if batch.relevant_to_main_phase(RenderPhase::Opaque3d) => {
            add_cached_or_rebuilt_phase(
                batch,
                RenderPhase::Opaque3d,
                build_context,
                command_cache,
                generation,
                commands,
                cache_stats,
                |batch, context, out| OpaqueBasePassProcessor.add_mesh_batch(batch, context, out),
            )
        }
        MeshDrawQueuePhase::AlphaMask if batch.relevant_to_main_phase(RenderPhase::AlphaMask3d) => {
            add_cached_or_rebuilt_phase(
                batch,
                RenderPhase::AlphaMask3d,
                build_context,
                command_cache,
                generation,
                commands,
                cache_stats,
                |batch, context, out| OpaqueBasePassProcessor.add_mesh_batch(batch, context, out),
            )
        }
        MeshDrawQueuePhase::Transparent => {}
        _ => {}
    }

    add_uncached_postprocess_phase(
        batch,
        build_context,
        commands,
        cache_stats,
        |batch, context, out| TaaReactiveMaskPassProcessor.add_mesh_batch(batch, context, out),
    );

    true
}

#[allow(clippy::too_many_arguments)]
fn add_cached_or_rebuilt_phase<R>(
    batch: &MeshBatchRef,
    phase: RenderPhase,
    build_context: &mut MeshPassBuildContext<'_, R>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    commands: &mut MeshDrawCommandList,
    cache_stats: &mut MeshDrawCommandCacheStats,
    add_batch: impl FnOnce(&MeshBatchRef, &mut MeshPassBuildContext<'_, R>, &mut MeshDrawCommandList),
) where
    R: MeshPipelineVariantResolver + ?Sized,
{
    if !CachedMeshDrawCommands::is_cacheable_batch_phase(batch, phase) {
        return;
    }
    let Some(key) = CachedMeshDrawKey::from_batch_phase(batch, phase) else {
        return;
    };
    match command_cache.lookup_status(&key, &batch.static_state, generation) {
        CachedMeshDrawLookup::Hit(command) => {
            cache_stats.cached_command_hit_count += 1;
            commands.push(command);
            return;
        }
        CachedMeshDrawLookup::Miss => {
            cache_stats.cache_miss_count += 1;
        }
        CachedMeshDrawLookup::Invalidated(invalidation) => {
            cache_stats.record_invalidation(invalidation);
        }
    }

    let mut rebuilt = MeshDrawCommandList::new();
    add_batch(batch, build_context, &mut rebuilt);
    for command in rebuilt.into_commands() {
        if command.phase != phase {
            continue;
        }
        command_cache.store(key, &batch.static_state, command.clone(), generation);
        cache_stats.command_rebuild_count += 1;
        commands.push(command);
    }
}

fn add_uncached_postprocess_phase<R>(
    batch: &MeshBatchRef,
    build_context: &mut MeshPassBuildContext<'_, R>,
    commands: &mut MeshDrawCommandList,
    cache_stats: &mut MeshDrawCommandCacheStats,
    add_batch: impl FnOnce(&MeshBatchRef, &mut MeshPassBuildContext<'_, R>, &mut MeshDrawCommandList),
) where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let command_start = commands.commands().len();
    add_batch(batch, build_context, commands);
    record_dynamic_commands_since(commands, command_start, cache_stats);
}

fn record_dynamic_commands_since(
    commands: &MeshDrawCommandList,
    command_start: usize,
    cache_stats: &mut MeshDrawCommandCacheStats,
) {
    let command_count = commands.commands().len().saturating_sub(command_start);
    cache_stats.command_rebuild_count += command_count;
    cache_stats.dynamic_command_count += command_count;
}
