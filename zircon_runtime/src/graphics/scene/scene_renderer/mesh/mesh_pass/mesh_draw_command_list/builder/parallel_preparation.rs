use std::sync::Arc;

use crate::core::framework::render::{
    RenderMeshStaticState, RenderPhase, ShaderQualityTier,
};
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::TaskPool;

use super::super::super::cached_mesh_draw_commands::{
    CachedMeshDrawCommands, CachedMeshDrawKey, CachedMeshDrawLookup, MeshDrawCommandCacheStats,
};
use super::super::super::mesh_pass_processor::{
    depth_prepass_command_spec, mesh_pass_command_specs, opaque_base_command_spec,
    shadow_command_spec, taa_reactive_mask_command_spec, MeshBatchRef, MeshPassCommandSpec,
};
use super::super::super::{MeshDrawCommand, MeshDrawCommandPayload, MeshPipelineVariantId};
use super::super::{MeshDrawCommandList, MeshPassCommandBuffers};
use super::super::super::super::mesh_draw::MeshDrawQueuePhase;
use super::super::super::super::mesh_pipeline_cache::MeshPipelineVariantResolver;
use super::parallel_admission::ParallelPreparationMode;
use super::{
    build_mesh_pass_command_buffers_from_ordered_batches_cached,
    collect_batches_in_source_order, record_preparation_result_profile,
};

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
    crate::profile_scope!("render", "mesh_commands", "prepare_cached_dispatch");
    let batches = collect_batches_in_source_order(batches);
    let preparation_mode = {
        crate::profile_scope!("render", "mesh_commands", "parallel_admission");
        ParallelPreparationMode::select(&batches, shader_quality, task_pool)
    };
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    crate::core::diagnostics::profiling::record_counter_batch(
        "render",
        &[
            ("mesh_commands.batch_count", batches.len() as f64),
            ("mesh_commands.worker_count", task_pool.parallelism() as f64),
            (
                "mesh_commands.parallel_enabled",
                u8::from(preparation_mode.is_parallel()) as f64,
            ),
            (
                "mesh_commands.dispatch_reason_code",
                preparation_mode.profile_code() as f64,
            ),
        ],
    );
    if !preparation_mode.is_parallel() {
        return build_mesh_pass_command_buffers_from_ordered_batches_cached(
            batches,
            variant_resolver,
            command_cache,
            generation,
            shader_quality,
        );
    }

    let plans = {
        crate::profile_scope!("render", "mesh_commands", "owner_transaction");
        batches
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
            .collect::<Vec<_>>()
    };
    let chunks = {
        crate::profile_scope!("render", "mesh_commands", "worker_projection_wait");
        task_pool.parallel_map_ordered(plans, build_prepared_batch_chunk)
    };

    let mut commands = MeshDrawCommandList::new();
    let mut cache_stats = MeshDrawCommandCacheStats::default();
    {
        crate::profile_scope!("render", "mesh_commands", "ordered_merge");
        for chunk in chunks {
            for store in chunk.cache_stores {
                command_cache.store(store.key, &store.state, store.payload, generation);
            }
            for command in chunk.commands {
                commands.push(command);
            }
            cache_stats.accumulate(chunk.cache_stats);
        }
    }
    record_preparation_result_profile(&commands, &cache_stats);
    {
        crate::profile_scope!("render", "mesh_commands", "seal_phase_buffers");
        MeshPassCommandBuffers::from_command_list(commands, cache_stats)
    }
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
    let Some(key) = CachedMeshDrawKey::from_batch_phase(&plan.batch, phase, shader_quality) else {
        return;
    };
    match command_cache.lookup_status(&key, &plan.batch.static_state, generation) {
        CachedMeshDrawLookup::Hit(payload) => {
            plan.cache_stats.cached_command_hit_count += 1;
            plan.commands.push(PreparedCommand::Cached(payload));
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
            PreparedCommand::Cached(payload) => {
                commands.push(plan.batch.project_cached_command(payload));
            }
            PreparedCommand::Build {
                spec,
                variant_id,
                cache_key,
            } => {
                let command = plan
                    .batch
                    .command(spec.phase, spec.pipeline_kind, variant_id);
                let command = if let Some(key) = cache_key {
                    let (command, payload) = command.into_shared_payload();
                    cache_stores.push(PreparedCacheStore {
                        key,
                        state: plan.batch.static_state,
                        payload,
                    });
                    command
                } else {
                    command
                };
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
    Cached(Arc<MeshDrawCommandPayload>),
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
    payload: Arc<MeshDrawCommandPayload>,
}
