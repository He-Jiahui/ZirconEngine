use crate::core::framework::render::{RenderPhase, RenderViewportPickPolicy, ShaderQualityTier};
use crate::core::TaskPool;

use super::super::super::mesh_draw::MeshDrawQueuePhase;
use super::super::super::mesh_pipeline_cache::MeshPipelineVariantResolver;
use super::super::super::MeshDraw;
use super::super::cached_mesh_draw_commands::{
    CachedMeshDrawCommands, CachedMeshDrawKey, CachedMeshDrawLookup, MeshDrawCommandCacheStats,
};
use super::super::mesh_pass_processor::{MeshBatchRef, MeshPassBuildContext, MeshPassProcessor};
use super::super::processors::{
    DepthPrepassProcessor, HitProxyPassProcessor, OpaqueBasePassProcessor, ShadowPassProcessor,
    TaaReactiveMaskPassProcessor, TransparentPassProcessor, VelocityPassProcessor,
};
use super::{MeshDrawCommandList, MeshPassCommandBuffers};

mod parallel_admission;
mod parallel_preparation;
#[cfg(test)]
#[path = "builder/tests/profiling_contract_tests.rs"]
mod profiling_contract_tests;

pub(super) use parallel_admission::should_prepare_batches_in_parallel;
pub(super) use parallel_preparation::build_mesh_pass_command_buffers_from_batches_cached_parallel;

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

/// Builds only the opaque command streams consumed by environment capture, avoiding variant
/// resolution for transparent, transmission, shadow, velocity, and temporal passes.
pub(crate) fn build_environment_capture_command_buffers<R>(
    draws: &[MeshDraw],
    variant_resolver: &mut R,
    shader_quality: ShaderQualityTier,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    build_environment_capture_command_buffers_from_batches(
        draws
            .iter()
            .enumerate()
            .map(|(draw_index, draw)| draw.mesh_pass_batch_ref(draw_index as u64, draw_index)),
        variant_resolver,
        shader_quality,
    )
}

pub(crate) fn build_hit_proxy_command_list<R>(
    draws: &[MeshDraw],
    variant_resolver: &mut R,
    policy: RenderViewportPickPolicy,
    shader_quality: ShaderQualityTier,
) -> MeshDrawCommandList
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let mut processor = HitProxyPassProcessor::new(policy);
    let mut commands = MeshDrawCommandList::new();
    let mut context = MeshPassBuildContext::new(variant_resolver, shader_quality);
    for (draw_index, draw) in draws.iter().enumerate() {
        let batch = draw.mesh_pass_batch_ref(draw_index as u64, draw_index);
        processor.add_mesh_batch(&batch, &mut context, &mut commands);
    }
    commands.sort();
    commands
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
    build_mesh_pass_command_buffers_from_ordered_batches_cached(
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

pub(crate) fn build_mesh_pass_command_buffers_cached_parallel<R>(
    draws: &[MeshDraw],
    variant_resolver: &mut R,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    shader_quality: ShaderQualityTier,
    task_pool: &TaskPool,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    build_mesh_pass_command_buffers_from_batches_cached_parallel(
        draws
            .iter()
            .enumerate()
            .map(|(draw_index, draw)| draw.mesh_pass_batch_ref(draw_index as u64, draw_index)),
        variant_resolver,
        command_cache,
        generation,
        shader_quality,
        task_pool,
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

pub(super) fn build_environment_capture_command_buffers_from_batches<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
    shader_quality: ShaderQualityTier,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let mut opaque_base = OpaqueBasePassProcessor;
    let mut commands = MeshDrawCommandList::new();
    let mut build_context = MeshPassBuildContext::new(variant_resolver, shader_quality);
    let mut cache_stats = MeshDrawCommandCacheStats::default();

    for batch in batches {
        let command_start = commands.commands().len();
        opaque_base.add_mesh_batch(&batch, &mut build_context, &mut commands);
        record_dynamic_commands_since(&commands, command_start, &mut cache_stats);
    }

    MeshPassCommandBuffers::from_command_list(commands, cache_stats)
}

fn build_mesh_pass_command_buffers_from_batches_uncached<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
    shader_quality: ShaderQualityTier,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let mut commands = MeshDrawCommandList::new();
    let mut build_context = MeshPassBuildContext::new(variant_resolver, shader_quality);
    let mut cache_stats = MeshDrawCommandCacheStats::default();

    for batch in batches {
        add_uncached_batch_commands(&batch, &mut build_context, &mut commands, &mut cache_stats);
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
    let batches = collect_batches_in_source_order(batches);
    build_mesh_pass_command_buffers_from_ordered_batches_cached(
        batches,
        variant_resolver,
        command_cache,
        generation,
        shader_quality,
    )
}

fn build_mesh_pass_command_buffers_from_ordered_batches_cached<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    shader_quality: ShaderQualityTier,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    crate::profile_scope!("render", "mesh_commands", "prepare_cached_serial");
    let mut commands = MeshDrawCommandList::new();
    let mut cache_stats = MeshDrawCommandCacheStats::default();
    cache_stats.record_resolver_configuration_invalidation(
        command_cache.synchronize_resolver_configuration(variant_resolver.configuration_epoch()),
    );
    let mut build_context = MeshPassBuildContext::new(variant_resolver, shader_quality);

    {
        crate::profile_scope!("render", "mesh_commands", "serial_prepare_and_project");
        for batch in batches {
            if add_cached_static_batch(
                &batch,
                &mut build_context,
                command_cache,
                generation,
                shader_quality,
                &mut commands,
                &mut cache_stats,
            ) {
                continue;
            }

            add_uncached_batch_commands(
                &batch,
                &mut build_context,
                &mut commands,
                &mut cache_stats,
            );
        }
    }

    record_preparation_result_profile(&commands, &cache_stats);
    {
        crate::profile_scope!("render", "mesh_commands", "seal_phase_buffers");
        MeshPassCommandBuffers::from_command_list(commands, cache_stats)
    }
}

fn record_preparation_result_profile(
    commands: &MeshDrawCommandList,
    cache_stats: &MeshDrawCommandCacheStats,
) {
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    crate::core::diagnostics::profiling::record_counter_batch(
        "render",
        &[
            (
                "mesh_commands.cache_hit_count",
                cache_stats.cached_command_hit_count as f64,
            ),
            (
                "mesh_commands.cache_miss_count",
                cache_stats.cache_miss_count as f64,
            ),
            (
                "mesh_commands.command_rebuild_count",
                cache_stats.command_rebuild_count as f64,
            ),
            (
                "mesh_commands.command_count",
                commands.commands().len() as f64,
            ),
        ],
    );
}

fn collect_batches_in_source_order(
    batches: impl IntoIterator<Item = MeshBatchRef>,
) -> Vec<MeshBatchRef> {
    crate::profile_scope!("render", "mesh_commands", "normalize_source_order");
    let mut batches = batches.into_iter().collect::<Vec<_>>();
    batches.sort_by_key(|batch| batch.source_draw_index);
    batches
}

fn add_cached_static_batch<R>(
    batch: &MeshBatchRef,
    build_context: &mut MeshPassBuildContext<'_, R>,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    shader_quality: ShaderQualityTier,
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
            shader_quality,
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
            shader_quality,
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
                shader_quality,
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
                shader_quality,
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
    shader_quality: ShaderQualityTier,
    commands: &mut MeshDrawCommandList,
    cache_stats: &mut MeshDrawCommandCacheStats,
    add_batch: impl FnOnce(&MeshBatchRef, &mut MeshPassBuildContext<'_, R>, &mut MeshDrawCommandList),
) where
    R: MeshPipelineVariantResolver + ?Sized,
{
    if !CachedMeshDrawCommands::is_cacheable_batch_phase(batch, phase) {
        return;
    }
    let Some(key) = CachedMeshDrawKey::from_batch_phase(batch, phase, shader_quality) else {
        return;
    };
    match command_cache.lookup_status(&key, &batch.static_state, generation) {
        CachedMeshDrawLookup::Hit(payload) => {
            cache_stats.cached_command_hit_count += 1;
            commands.push(batch.project_cached_command(payload));
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
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    let rebuilt_command_count = rebuilt.commands().len();
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    let (rebuilt_grow_count, rebuilt_peak_capacity) = rebuilt.staging_arena_cost();
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    let mut merged_command_count = 0;
    for command in rebuilt.into_commands() {
        if command.phase != phase {
            continue;
        }
        let (command, payload) = command.into_shared_payload();
        command_cache.store(key, &batch.static_state, payload, generation);
        cache_stats.command_rebuild_count += 1;
        #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
        {
            merged_command_count += 1;
        }
        commands.push(command);
    }
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    commands.record_staging_merge(merged_command_count, rebuilt_command_count);
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    commands.record_staging_arena_cost(rebuilt_grow_count, rebuilt_peak_capacity);
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

fn add_uncached_batch_commands<R>(
    batch: &MeshBatchRef,
    build_context: &mut MeshPassBuildContext<'_, R>,
    commands: &mut MeshDrawCommandList,
    cache_stats: &mut MeshDrawCommandCacheStats,
) where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let command_start = commands.commands().len();
    DepthPrepassProcessor.add_mesh_batch(batch, build_context, commands);
    ShadowPassProcessor.add_mesh_batch(batch, build_context, commands);
    OpaqueBasePassProcessor.add_mesh_batch(batch, build_context, commands);
    TransparentPassProcessor.add_mesh_batch(batch, build_context, commands);
    VelocityPassProcessor.add_mesh_batch(batch, build_context, commands);
    TaaReactiveMaskPassProcessor.add_mesh_batch(batch, build_context, commands);
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
