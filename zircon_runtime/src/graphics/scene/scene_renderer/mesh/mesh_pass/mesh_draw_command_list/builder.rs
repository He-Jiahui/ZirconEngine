use std::collections::HashSet;

use rayon::prelude::*;

use crate::core::TaskPool;
use crate::core::framework::render::{RenderMeshStaticState, RenderPhase, ShaderQualityTier};

use super::super::super::MeshDraw;
use super::super::super::mesh_draw::MeshDrawQueuePhase;
use super::super::super::mesh_pipeline_cache::MeshPipelineVariantResolver;
use super::super::cached_mesh_draw_commands::{
    CachedMeshDrawCommands, CachedMeshDrawKey, CachedMeshDrawLookup, MeshDrawCommandCacheStats,
};
use super::super::mesh_pass_processor::{
    MeshBatchRef, MeshPassBuildContext, MeshPassCommandSpec, MeshPassProcessor,
    depth_prepass_command_spec, mesh_pass_command_specs, opaque_base_command_spec,
    shadow_command_spec, taa_reactive_mask_command_spec,
};
use super::super::processors::{
    DepthPrepassProcessor, OpaqueBasePassProcessor, ShadowPassProcessor,
    TaaReactiveMaskPassProcessor, TransparentPassProcessor, VelocityPassProcessor,
};
use super::super::{MeshDrawCommand, MeshPipelineVariantId};
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

pub(super) fn build_mesh_pass_command_buffers_from_batches_cached_parallel<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    shader_quality: ShaderQualityTier,
    task_pool: &TaskPool,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let mut batches = batches.into_iter().collect::<Vec<_>>();
    batches.sort_by_key(|batch| batch.source_draw_index);
    if task_pool.parallelism() <= 1 || batches.len() < 2 || has_duplicate_cache_keys(&batches) {
        return build_mesh_pass_command_buffers_from_batches_cached(
            batches,
            variant_resolver,
            command_cache,
            generation,
            shader_quality,
        );
    }

    let plans = batches
        .into_iter()
        .map(|batch| {
            prepare_batch_plan(
                batch,
                variant_resolver,
                command_cache,
                generation,
                shader_quality,
            )
        })
        .collect::<Vec<_>>();
    let chunks = task_pool.install(|| {
        plans
            .into_par_iter()
            .map(build_prepared_batch_chunk)
            .collect::<Vec<_>>()
    });

    let mut commands = MeshDrawCommandList::new();
    let mut cache_stats = MeshDrawCommandCacheStats::default();
    for chunk in chunks {
        for store in chunk.cache_stores {
            command_cache.store(store.key, &store.state, store.command, generation);
        }
        for command in chunk.commands {
            commands.push(command);
        }
        cache_stats.accumulate(chunk.cache_stats);
    }
    MeshPassCommandBuffers::from_command_list(commands, cache_stats)
}

fn prepare_batch_plan<R>(
    batch: MeshBatchRef,
    variant_resolver: &mut R,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    shader_quality: ShaderQualityTier,
) -> PreparedBatchPlan
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let mut plan = PreparedBatchPlan::new(batch);
    if plan.batch.static_state.has_authoritative_revisions()
        && plan.batch.queue_profile.static_batch_eligible()
        && plan.batch.cache_identity.is_some()
        && !plan.batch.pipeline_key.requires_forward_path()
    {
        if plan.batch.queue_profile.early_z_eligible()
            && plan.batch.relevant_to_main_phase(RenderPhase::Prepass)
        {
            let spec = depth_prepass_command_spec(&plan.batch);
            prepare_cached_phase(
                &mut plan,
                RenderPhase::Prepass,
                spec,
                variant_resolver,
                command_cache,
                generation,
                shader_quality,
            );
        }
        if plan.batch.casts_shadow && plan.batch.relevant_to_shadow_view() {
            let spec = shadow_command_spec(&plan.batch);
            prepare_cached_phase(
                &mut plan,
                RenderPhase::Shadow,
                spec,
                variant_resolver,
                command_cache,
                generation,
                shader_quality,
            );
        }
        let main_phase = match plan.batch.phase() {
            MeshDrawQueuePhase::Opaque
                if plan.batch.relevant_to_main_phase(RenderPhase::Opaque3d) =>
            {
                Some(RenderPhase::Opaque3d)
            }
            MeshDrawQueuePhase::AlphaMask
                if plan.batch.relevant_to_main_phase(RenderPhase::AlphaMask3d) =>
            {
                Some(RenderPhase::AlphaMask3d)
            }
            _ => None,
        };
        if let Some(phase) = main_phase {
            let spec = opaque_base_command_spec(&plan.batch);
            prepare_cached_phase(
                &mut plan,
                phase,
                spec,
                variant_resolver,
                command_cache,
                generation,
                shader_quality,
            );
        }
        if let Some(spec) = taa_reactive_mask_command_spec(&plan.batch) {
            prepare_dynamic_command(&mut plan, spec, variant_resolver, shader_quality);
        }
        return plan;
    }

    for spec in mesh_pass_command_specs(&plan.batch).into_iter().flatten() {
        prepare_dynamic_command(&mut plan, spec, variant_resolver, shader_quality);
    }
    plan
}

#[allow(clippy::too_many_arguments)]
fn prepare_cached_phase<R>(
    plan: &mut PreparedBatchPlan,
    phase: RenderPhase,
    spec: Option<MeshPassCommandSpec>,
    variant_resolver: &mut R,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
    shader_quality: ShaderQualityTier,
) where
    R: MeshPipelineVariantResolver + ?Sized,
{
    if !CachedMeshDrawCommands::is_cacheable_batch_phase(&plan.batch, phase) {
        return;
    }
    let Some(key) = CachedMeshDrawKey::from_batch_phase(&plan.batch, phase) else {
        return;
    };
    match command_cache.lookup_status(&key, &plan.batch.static_state, generation) {
        CachedMeshDrawLookup::Hit(command) => {
            plan.cache_stats.cached_command_hit_count += 1;
            plan.commands.push(PreparedCommand::Cached(command));
            return;
        }
        CachedMeshDrawLookup::Miss => plan.cache_stats.cache_miss_count += 1,
        CachedMeshDrawLookup::Invalidated(invalidation) => {
            plan.cache_stats.record_invalidation(invalidation);
        }
    }
    let Some(spec) = spec else {
        return;
    };
    let variant_id = resolve_spec_variant(variant_resolver, &plan.batch, spec, shader_quality);
    plan.commands.push(PreparedCommand::Build {
        spec,
        variant_id,
        cache_key: Some(key),
    });
    plan.cache_stats.command_rebuild_count += 1;
}

fn prepare_dynamic_command<R>(
    plan: &mut PreparedBatchPlan,
    spec: MeshPassCommandSpec,
    variant_resolver: &mut R,
    shader_quality: ShaderQualityTier,
) where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let variant_id = resolve_spec_variant(variant_resolver, &plan.batch, spec, shader_quality);
    plan.commands.push(PreparedCommand::Build {
        spec,
        variant_id,
        cache_key: None,
    });
    plan.cache_stats.command_rebuild_count += 1;
    plan.cache_stats.dynamic_command_count += 1;
}

fn resolve_spec_variant<R>(
    variant_resolver: &mut R,
    batch: &MeshBatchRef,
    spec: MeshPassCommandSpec,
    shader_quality: ShaderQualityTier,
) -> MeshPipelineVariantId
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    variant_resolver.resolve_variant_for_geometry(
        spec.pipeline_kind,
        &batch.pipeline_key,
        batch.queue_profile.shader_geometry_source_id(),
        shader_quality,
    )
}

fn build_prepared_batch_chunk(plan: PreparedBatchPlan) -> PreparedBatchChunk {
    let mut commands = Vec::with_capacity(plan.commands.len());
    let mut cache_stores = Vec::new();
    for prepared in plan.commands {
        match prepared {
            PreparedCommand::Cached(command) => commands.push(command),
            PreparedCommand::Build {
                spec,
                variant_id,
                cache_key,
            } => {
                let command = plan
                    .batch
                    .command(spec.phase, spec.pipeline_kind, variant_id);
                if let Some(key) = cache_key {
                    cache_stores.push(PreparedCacheStore {
                        key,
                        state: plan.batch.static_state,
                        command: command.clone(),
                    });
                }
                commands.push(command);
            }
        }
    }
    PreparedBatchChunk {
        commands,
        cache_stores,
        cache_stats: plan.cache_stats,
    }
}

fn has_duplicate_cache_keys(batches: &[MeshBatchRef]) -> bool {
    let mut keys = HashSet::new();
    for batch in batches {
        for phase in [
            RenderPhase::Prepass,
            RenderPhase::Shadow,
            RenderPhase::Opaque3d,
            RenderPhase::AlphaMask3d,
        ] {
            if !CachedMeshDrawCommands::is_cacheable_batch_phase(batch, phase) {
                continue;
            }
            if let Some(key) = CachedMeshDrawKey::from_batch_phase(batch, phase) {
                if !keys.insert(key) {
                    return true;
                }
            }
        }
    }
    false
}

struct PreparedBatchPlan {
    batch: MeshBatchRef,
    commands: Vec<PreparedCommand>,
    cache_stats: MeshDrawCommandCacheStats,
}

impl PreparedBatchPlan {
    fn new(batch: MeshBatchRef) -> Self {
        Self {
            batch,
            commands: Vec::with_capacity(6),
            cache_stats: MeshDrawCommandCacheStats::default(),
        }
    }
}

enum PreparedCommand {
    Cached(MeshDrawCommand),
    Build {
        spec: MeshPassCommandSpec,
        variant_id: MeshPipelineVariantId,
        cache_key: Option<CachedMeshDrawKey>,
    },
}

struct PreparedBatchChunk {
    commands: Vec<MeshDrawCommand>,
    cache_stores: Vec<PreparedCacheStore>,
    cache_stats: MeshDrawCommandCacheStats,
}

struct PreparedCacheStore {
    key: CachedMeshDrawKey,
    state: RenderMeshStaticState,
    command: MeshDrawCommand,
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
