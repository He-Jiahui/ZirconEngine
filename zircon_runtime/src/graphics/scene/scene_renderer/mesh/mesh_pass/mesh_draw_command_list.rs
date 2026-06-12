use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};

use super::super::mesh_draw::MeshDraw;
use super::super::mesh_pipeline_cache::MeshPipelineVariantResolver;
use super::cached_mesh_draw_commands::{
    CachedMeshDrawCommands, CachedMeshDrawKey, MeshDrawCommandCacheStats,
};
use super::indirect_draw_batcher::{IndirectDrawBatcher, IndirectDrawBatcherStats};
use super::mesh_draw_command::{DrawInstanceSource, MeshDrawArgs, MeshDrawCommand};
use super::mesh_pass_processor::{MeshBatchRef, MeshPassBuildContext, MeshPassProcessor};
use super::processors::{
    DepthPrepassProcessor, OpaqueBasePassProcessor, ShadowPassProcessor, TransparentPassProcessor,
    VelocityPassProcessor,
};

#[derive(Clone, Default)]
pub(crate) struct MeshDrawCommandList {
    commands: Vec<MeshDrawCommand>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshDrawCommandListStats {
    pub(crate) command_count: usize,
    pub(crate) direct_indexed_count: usize,
    pub(crate) indirect_indexed_count: usize,
    pub(crate) gpu_scene_instance_count: usize,
}

#[derive(Clone, Default)]
pub(crate) struct MeshPassCommandBuffers {
    depth_prepass: MeshDrawCommandList,
    shadow: MeshDrawCommandList,
    opaque: MeshDrawCommandList,
    alpha_mask: MeshDrawCommandList,
    transparent: MeshDrawCommandList,
    velocity: MeshDrawCommandList,
    cache_stats: MeshDrawCommandCacheStats,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshPassCommandBufferStats {
    pub(crate) command_count: usize,
    pub(crate) depth_prepass_command_count: usize,
    pub(crate) shadow_command_count: usize,
    pub(crate) opaque_command_count: usize,
    pub(crate) alpha_mask_command_count: usize,
    pub(crate) transparent_command_count: usize,
    pub(crate) velocity_command_count: usize,
    pub(crate) direct_indexed_count: usize,
    pub(crate) indirect_indexed_count: usize,
    pub(crate) gpu_scene_instance_count: usize,
    pub(crate) cached_command_hit_count: usize,
    pub(crate) command_rebuild_count: usize,
    pub(crate) dynamic_command_count: usize,
    pub(crate) indirect_batch_count: usize,
    pub(crate) indirect_batched_draw_count: usize,
    pub(crate) indirect_fallback_draw_count: usize,
    pub(crate) indirect_args_count: usize,
}

impl MeshDrawCommandList {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn from_commands(mut commands: Vec<MeshDrawCommand>) -> Self {
        sort_mesh_draw_commands(&mut commands);
        Self { commands }
    }

    pub(crate) fn push(&mut self, command: MeshDrawCommand) {
        self.commands.push(command);
    }

    pub(crate) fn sort(&mut self) {
        sort_mesh_draw_commands(&mut self.commands);
    }

    pub(crate) fn commands(&self) -> &[MeshDrawCommand] {
        &self.commands
    }

    pub(crate) fn into_commands(self) -> Vec<MeshDrawCommand> {
        self.commands
    }

    pub(crate) fn iter_phase(&self, phase: RenderPhase) -> impl Iterator<Item = &MeshDrawCommand> {
        self.commands
            .iter()
            .filter(move |command| command.phase == phase)
    }

    pub(crate) fn stats(&self) -> MeshDrawCommandListStats {
        summarize_mesh_draw_commands(&self.commands)
    }
}

impl MeshPassCommandBuffers {
    fn from_command_list(
        commands: MeshDrawCommandList,
        cache_stats: MeshDrawCommandCacheStats,
    ) -> Self {
        let mut depth_prepass = Vec::new();
        let mut shadow = Vec::new();
        let mut opaque = Vec::new();
        let mut alpha_mask = Vec::new();
        let mut transparent = Vec::new();
        let mut velocity = Vec::new();

        for command in commands.into_commands() {
            match command.phase {
                RenderPhase::Prepass => depth_prepass.push(command),
                RenderPhase::Shadow => shadow.push(command),
                RenderPhase::Opaque3d => opaque.push(command),
                RenderPhase::AlphaMask3d => alpha_mask.push(command),
                RenderPhase::Transparent3d => transparent.push(command),
                RenderPhase::PostProcess => velocity.push(command),
                _ => {}
            }
        }

        Self {
            depth_prepass: MeshDrawCommandList::from_commands(depth_prepass),
            shadow: MeshDrawCommandList::from_commands(shadow),
            opaque: MeshDrawCommandList::from_commands(opaque),
            alpha_mask: MeshDrawCommandList::from_commands(alpha_mask),
            transparent: MeshDrawCommandList::from_commands(transparent),
            velocity: MeshDrawCommandList::from_commands(velocity),
            cache_stats,
        }
    }

    pub(crate) fn depth_prepass(&self) -> &MeshDrawCommandList {
        &self.depth_prepass
    }

    pub(crate) fn shadow(&self) -> &MeshDrawCommandList {
        &self.shadow
    }

    pub(crate) fn opaque(&self) -> &MeshDrawCommandList {
        &self.opaque
    }

    pub(crate) fn alpha_mask(&self) -> &MeshDrawCommandList {
        &self.alpha_mask
    }

    pub(crate) fn transparent(&self) -> &MeshDrawCommandList {
        &self.transparent
    }

    pub(crate) fn velocity(&self) -> &MeshDrawCommandList {
        &self.velocity
    }

    pub(crate) fn stats(&self) -> MeshPassCommandBufferStats {
        self.stats_with_indirect_batches(&RenderCapabilitySummary::default())
    }

    pub(crate) fn stats_with_indirect_batches(
        &self,
        capabilities: &RenderCapabilitySummary,
    ) -> MeshPassCommandBufferStats {
        let depth_prepass = self.depth_prepass.stats();
        let shadow = self.shadow.stats();
        let opaque = self.opaque.stats();
        let alpha_mask = self.alpha_mask.stats();
        let transparent = self.transparent.stats();
        let velocity = self.velocity.stats();
        let lists = [
            depth_prepass,
            shadow,
            opaque,
            alpha_mask,
            transparent,
            velocity,
        ];

        MeshPassCommandBufferStats {
            command_count: lists.iter().map(|stats| stats.command_count).sum(),
            depth_prepass_command_count: depth_prepass.command_count,
            shadow_command_count: shadow.command_count,
            opaque_command_count: opaque.command_count,
            alpha_mask_command_count: alpha_mask.command_count,
            transparent_command_count: transparent.command_count,
            velocity_command_count: velocity.command_count,
            direct_indexed_count: lists.iter().map(|stats| stats.direct_indexed_count).sum(),
            indirect_indexed_count: lists.iter().map(|stats| stats.indirect_indexed_count).sum(),
            gpu_scene_instance_count: lists
                .iter()
                .map(|stats| stats.gpu_scene_instance_count)
                .sum(),
            cached_command_hit_count: self.cache_stats.cached_command_hit_count,
            command_rebuild_count: self.cache_stats.command_rebuild_count,
            dynamic_command_count: self.cache_stats.dynamic_command_count,
            ..indirect_batch_stats(
                capabilities,
                [
                    self.depth_prepass.commands(),
                    self.shadow.commands(),
                    self.opaque.commands(),
                    self.alpha_mask.commands(),
                    self.transparent.commands(),
                    self.velocity.commands(),
                ],
            )
        }
    }
}

fn indirect_batch_stats<const N: usize>(
    capabilities: &RenderCapabilitySummary,
    command_lists: [&[MeshDrawCommand]; N],
) -> MeshPassCommandBufferStats {
    let mut stats = MeshPassCommandBufferStats::default();
    for commands in command_lists {
        let batcher = IndirectDrawBatcher::build(commands, capabilities);
        let batch_stats = batcher.stats();
        accumulate_indirect_batch_stats(&mut stats, batch_stats);
    }
    stats
}

fn accumulate_indirect_batch_stats(
    stats: &mut MeshPassCommandBufferStats,
    batch_stats: IndirectDrawBatcherStats,
) {
    stats.indirect_batch_count += batch_stats.batch_count;
    stats.indirect_batched_draw_count += batch_stats.batched_draw_count;
    stats.indirect_fallback_draw_count += batch_stats.fallback_draw_count;
    stats.indirect_args_count += batch_stats.indirect_args_count;
}

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
    )
}

pub(crate) fn build_mesh_pass_command_buffers_cached<R>(
    draws: &[MeshDraw],
    variant_resolver: &mut R,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
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
    )
}

fn build_mesh_pass_command_buffers_from_batches<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    build_mesh_pass_command_buffers_from_batches_uncached(batches, variant_resolver)
}

fn build_mesh_pass_command_buffers_from_batches_uncached<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let mut depth_prepass = DepthPrepassProcessor;
    let mut opaque_base = OpaqueBasePassProcessor;
    let mut transparent = TransparentPassProcessor;
    let mut shadow = ShadowPassProcessor;
    let mut velocity = VelocityPassProcessor;
    let mut commands = MeshDrawCommandList::new();
    let mut build_context = MeshPassBuildContext::new(variant_resolver);
    let mut cache_stats = MeshDrawCommandCacheStats::default();

    for batch in batches {
        let mut batch_commands = MeshDrawCommandList::new();
        depth_prepass.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        shadow.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        opaque_base.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        transparent.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        velocity.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        append_dynamic_commands(batch_commands, &mut commands, &mut cache_stats);
    }

    commands.sort();
    MeshPassCommandBuffers::from_command_list(commands, cache_stats)
}

fn build_mesh_pass_command_buffers_from_batches_cached<R>(
    batches: impl IntoIterator<Item = MeshBatchRef>,
    variant_resolver: &mut R,
    command_cache: &mut CachedMeshDrawCommands,
    generation: u64,
) -> MeshPassCommandBuffers
where
    R: MeshPipelineVariantResolver + ?Sized,
{
    let mut depth_prepass = DepthPrepassProcessor;
    let mut opaque_base = OpaqueBasePassProcessor;
    let mut transparent = TransparentPassProcessor;
    let mut shadow = ShadowPassProcessor;
    let mut velocity = VelocityPassProcessor;
    let mut commands = MeshDrawCommandList::new();
    let mut build_context = MeshPassBuildContext::new(variant_resolver);
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

        let mut batch_commands = MeshDrawCommandList::new();
        depth_prepass.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        shadow.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        opaque_base.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        transparent.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        velocity.add_mesh_batch(&batch, &mut build_context, &mut batch_commands);
        append_dynamic_commands(batch_commands, &mut commands, &mut cache_stats);
    }

    commands.sort();
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
        super::super::mesh_draw::MeshDrawQueuePhase::Opaque
            if batch.relevant_to_main_phase(RenderPhase::Opaque3d) =>
        {
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
        super::super::mesh_draw::MeshDrawQueuePhase::AlphaMask
            if batch.relevant_to_main_phase(RenderPhase::AlphaMask3d) =>
        {
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
        super::super::mesh_draw::MeshDrawQueuePhase::Transparent => {}
        _ => {}
    }

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
    if let Some(command) = command_cache.lookup(&key, &batch.static_state, generation) {
        cache_stats.cached_command_hit_count += 1;
        commands.push(command);
        return;
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

fn append_dynamic_commands(
    batch_commands: MeshDrawCommandList,
    commands: &mut MeshDrawCommandList,
    cache_stats: &mut MeshDrawCommandCacheStats,
) {
    for command in batch_commands.into_commands() {
        cache_stats.command_rebuild_count += 1;
        cache_stats.dynamic_command_count += 1;
        commands.push(command);
    }
}

fn sort_mesh_draw_commands(commands: &mut [MeshDrawCommand]) {
    commands.sort_by_key(|command| {
        (
            command.phase.queue_order(),
            command.sort_key,
            command.pipeline_variant_id.value(),
        )
    });
}

fn summarize_mesh_draw_commands(commands: &[MeshDrawCommand]) -> MeshDrawCommandListStats {
    let mut stats = MeshDrawCommandListStats::default();
    for command in commands {
        stats.command_count += 1;
        match &command.draw_args {
            MeshDrawArgs::DirectIndexed { .. } => stats.direct_indexed_count += 1,
            MeshDrawArgs::IndexedIndirect { .. } => stats.indirect_indexed_count += 1,
        }
        let DrawInstanceSource::GpuSceneInstance {
            first_instance_index,
            instance_count,
        } = &command.instance_source;
        debug_assert!(first_instance_index.checked_add(*instance_count).is_some());
        stats.gpu_scene_instance_count += *instance_count as usize;
    }
    stats
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderCapabilitySummary, RenderMeshStaticState, RenderPhase,
    };
    use crate::core::framework::scene::Mobility;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
        MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::MeshPipelineVariantRegistry;

    use super::*;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshBatchRef, MeshBindHandle, MeshDrawArgs, MeshDrawCommand,
        MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
    };

    #[test]
    fn mesh_draw_command_list_sorts_by_phase_then_sort_key() {
        let commands = MeshDrawCommandList::from_commands(vec![
            command(RenderPhase::Transparent3d, 10, 3),
            command(RenderPhase::Opaque3d, 50, 1),
            command(RenderPhase::Opaque3d, 20, 2),
            command(RenderPhase::Shadow, 90, 4),
        ]);

        let ordered = commands
            .commands()
            .iter()
            .map(|command| (command.phase, command.sort_key))
            .collect::<Vec<_>>();

        assert_eq!(
            ordered,
            vec![
                (RenderPhase::Shadow, 90),
                (RenderPhase::Opaque3d, 20),
                (RenderPhase::Opaque3d, 50),
                (RenderPhase::Transparent3d, 10),
            ]
        );
    }

    #[test]
    fn mesh_draw_command_list_reports_draw_and_instance_sources() {
        let commands = MeshDrawCommandList::from_commands(vec![
            command(RenderPhase::Opaque3d, 1, 1),
            MeshDrawCommand::new(
                RenderPhase::Opaque3d,
                MeshPassPipelineKind::Base,
                default_pipeline_key(),
                MeshPipelineVariantId::new(2),
                2,
                DrawInstanceSource::GpuSceneInstance {
                    first_instance_index: 7,
                    instance_count: 1,
                },
                MeshGeometryHandle::test(2),
                MeshDrawArgs::test_indexed_indirect(9, 64),
            ),
        ]);

        assert_eq!(
            commands.stats(),
            MeshDrawCommandListStats {
                command_count: 2,
                direct_indexed_count: 1,
                indirect_indexed_count: 1,
                gpu_scene_instance_count: 2,
            }
        );
    }

    #[test]
    fn mesh_batch_ref_emits_gpu_scene_instance_command() {
        let command = batch(MeshDrawQueuePhase::Opaque, 10)
            .with_gpu_scene_instance_span(7, 2)
            .command(
                RenderPhase::Opaque3d,
                MeshPassPipelineKind::Base,
                MeshPipelineVariantId::new(4),
            );

        match &command.instance_source {
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index,
                instance_count,
            } => {
                assert_eq!(*first_instance_index, 7);
                assert_eq!(*instance_count, 2);
            }
        }
        match &command.draw_args {
            MeshDrawArgs::DirectIndexed {
                first_instance,
                instance_count,
                ..
            } => {
                assert_eq!(*first_instance, 7);
                assert_eq!(*instance_count, 2);
            }
            MeshDrawArgs::IndexedIndirect { .. } => panic!("expected direct indexed draw args"),
        }
    }

    #[test]
    fn mesh_draw_command_list_filters_phase_views_without_resorting() {
        let commands = MeshDrawCommandList::from_commands(vec![
            command(RenderPhase::Opaque3d, 20, 1),
            command(RenderPhase::Transparent3d, 10, 2),
            command(RenderPhase::Opaque3d, 30, 3),
        ]);

        let opaque_sort_keys = commands
            .iter_phase(RenderPhase::Opaque3d)
            .map(|command| command.sort_key)
            .collect::<Vec<_>>();

        assert_eq!(opaque_sort_keys, vec![20, 30]);
    }

    #[test]
    fn mesh_pass_command_buffers_build_expected_phase_counts_from_batches() {
        let mut variants = MeshPipelineVariantRegistry::default();
        let buffers = build_mesh_pass_command_buffers_from_batches(
            [
                batch(MeshDrawQueuePhase::Opaque, 10)
                    .with_source_draw_index(0)
                    .with_casts_shadow(true)
                    .with_previous_motion_vector_transform(true),
                batch(MeshDrawQueuePhase::AlphaMask, 20)
                    .with_source_draw_index(1)
                    .with_casts_shadow(true),
                batch(MeshDrawQueuePhase::Transparent, 30).with_source_draw_index(2),
                batch(MeshDrawQueuePhase::Opaque, 40).with_source_draw_index(3),
            ],
            &mut variants,
        );

        assert_eq!(buffers.depth_prepass().commands().len(), 3);
        assert_eq!(buffers.shadow().commands().len(), 2);
        assert_eq!(buffers.opaque().commands().len(), 2);
        assert_eq!(buffers.alpha_mask().commands().len(), 1);
        assert_eq!(buffers.transparent().commands().len(), 1);
        assert_eq!(buffers.velocity().commands().len(), 1);
        assert_eq!(
            buffers
                .opaque()
                .commands()
                .iter()
                .map(|command| command.source_draw_index)
                .collect::<Vec<_>>(),
            vec![0, 3]
        );
        assert_eq!(
            buffers.stats(),
            MeshPassCommandBufferStats {
                command_count: 10,
                depth_prepass_command_count: 3,
                shadow_command_count: 2,
                opaque_command_count: 2,
                alpha_mask_command_count: 1,
                transparent_command_count: 1,
                velocity_command_count: 1,
                direct_indexed_count: 10,
                indirect_indexed_count: 0,
                gpu_scene_instance_count: 10,
                cached_command_hit_count: 0,
                command_rebuild_count: 10,
                dynamic_command_count: 10,
                indirect_batch_count: 0,
                indirect_batched_draw_count: 0,
                indirect_fallback_draw_count: 10,
                indirect_args_count: 0,
            }
        );
    }

    #[test]
    fn mesh_pass_command_buffers_report_indirect_batch_stats_when_gpu_driven_supported() {
        let mut variants = MeshPipelineVariantRegistry::default();
        let buffers = build_mesh_pass_command_buffers_from_batches(
            [
                batch(MeshDrawQueuePhase::Opaque, 10),
                batch(MeshDrawQueuePhase::Opaque, 10),
            ],
            &mut variants,
        );

        let default_stats = buffers.stats();
        assert_eq!(default_stats.indirect_batch_count, 0);
        assert_eq!(default_stats.indirect_fallback_draw_count, 4);

        let stats = buffers.stats_with_indirect_batches(&gpu_driven_capabilities());

        assert_eq!(stats.command_count, 4);
        assert_eq!(stats.indirect_batch_count, 2);
        assert_eq!(stats.indirect_batched_draw_count, 4);
        assert_eq!(stats.indirect_fallback_draw_count, 0);
        assert_eq!(stats.indirect_args_count, 4);
    }

    #[test]
    fn mesh_pass_command_buffers_assign_cache_variants_by_pipeline_kind() {
        let mut variants = MeshPipelineVariantRegistry::default();
        let buffers = build_mesh_pass_command_buffers_from_batches(
            [batch(MeshDrawQueuePhase::Opaque, 10)
                .with_casts_shadow(true)
                .with_previous_motion_vector_transform(true)],
            &mut variants,
        );

        let depth = buffers.depth_prepass().commands()[0].pipeline_variant_id;
        let shadow = buffers.shadow().commands()[0].pipeline_variant_id;
        let opaque = buffers.opaque().commands()[0].pipeline_variant_id;
        let velocity = buffers.velocity().commands()[0].pipeline_variant_id;

        assert_eq!(depth, MeshPipelineVariantId::new(0));
        assert_eq!(shadow, MeshPipelineVariantId::new(0));
        assert_ne!(opaque, MeshPipelineVariantId::new(0));
        assert_ne!(velocity, MeshPipelineVariantId::new(0));
        assert_ne!(opaque, velocity);
        assert_eq!(variants.len(), 2);
    }

    #[test]
    fn mesh_pass_command_buffers_reuse_static_cached_commands_on_second_frame() {
        let mut variants = MeshPipelineVariantRegistry::default();
        let mut cache = CachedMeshDrawCommands::default();
        let static_state = RenderMeshStaticState::new(true, 11, 17);
        let batch = static_batch(MeshDrawQueuePhase::Opaque, 10)
            .with_cache_identity(7, 0)
            .with_static_state(static_state)
            .with_casts_shadow(true);

        let first = build_mesh_pass_command_buffers_from_batches_cached(
            [batch.clone()],
            &mut variants,
            &mut cache,
            1,
        )
        .stats();
        let second = build_mesh_pass_command_buffers_from_batches_cached(
            [batch],
            &mut variants,
            &mut cache,
            2,
        )
        .stats();

        assert_eq!(first.command_count, 3);
        assert_eq!(first.cached_command_hit_count, 0);
        assert_eq!(first.command_rebuild_count, 3);
        assert_eq!(second.command_count, 3);
        assert_eq!(second.cached_command_hit_count, 3);
        assert_eq!(second.command_rebuild_count, 0);
        assert_eq!(second.dynamic_command_count, 0);
    }

    fn command(phase: RenderPhase, sort_key: u64, variant_id: u32) -> MeshDrawCommand {
        MeshDrawCommand::new(
            phase,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(variant_id),
            sort_key,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: variant_id,
                instance_count: 1,
            },
            MeshGeometryHandle::test(u64::from(variant_id)),
            MeshDrawArgs::direct_indexed(0, 3).with_instance_span(variant_id, 1),
        )
    }

    fn batch(phase: MeshDrawQueuePhase, sort_key: u64) -> MeshBatchRef {
        MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                phase,
                MeshDrawGeometrySource::Prepared,
                Mobility::Dynamic,
                false,
                false,
                false,
            ),
            default_pipeline_key(),
            sort_key,
            MeshGeometryHandle::test(sort_key),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_gpu_scene_instance_span(sort_key as u32, 1)
        .with_material_textures(MeshBindHandle::test(sort_key + 100))
        .with_material(MeshBindHandle::test(sort_key + 200))
        .with_standard_material(MeshBindHandle::test(sort_key + 300))
    }

    fn static_batch(phase: MeshDrawQueuePhase, sort_key: u64) -> MeshBatchRef {
        MeshBatchRef::new(
            MeshDrawQueueProfile::new(
                phase,
                MeshDrawGeometrySource::Prepared,
                Mobility::Static,
                false,
                false,
                false,
            ),
            default_pipeline_key(),
            sort_key,
            MeshGeometryHandle::test(sort_key),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_gpu_scene_instance_span(sort_key as u32, 1)
        .with_material_textures(MeshBindHandle::test(sort_key + 100))
        .with_material(MeshBindHandle::test(sort_key + 200))
        .with_standard_material(MeshBindHandle::test(sort_key + 300))
    }

    fn gpu_driven_capabilities() -> RenderCapabilitySummary {
        RenderCapabilitySummary {
            supports_indirect_draw: true,
            supports_multi_draw_indirect: true,
            supports_indirect_first_instance: true,
            ..RenderCapabilitySummary::default()
        }
    }
}
