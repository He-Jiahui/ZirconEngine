use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};

use super::cached_mesh_draw_commands::MeshDrawCommandCacheStats;
use super::indirect_draw_batcher::{IndirectDrawBatcher, IndirectDrawBatcherStats};
use super::mesh_draw_command::{DrawInstanceSource, MeshDrawArgs, MeshDrawCommand};

mod builder;
#[cfg(test)]
mod tests;

pub(crate) use builder::{build_mesh_pass_command_buffers, build_mesh_pass_command_buffers_cached};

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
    advanced_pbr_opaque: MeshDrawCommandList,
    transmission: MeshDrawCommandList,
    transparent: MeshDrawCommandList,
    velocity: MeshDrawCommandList,
    taa_reactive_mask: MeshDrawCommandList,
    cache_stats: MeshDrawCommandCacheStats,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshPassCommandBufferStats {
    pub(crate) command_count: usize,
    pub(crate) depth_prepass_command_count: usize,
    pub(crate) shadow_command_count: usize,
    pub(crate) opaque_command_count: usize,
    pub(crate) alpha_mask_command_count: usize,
    pub(crate) advanced_pbr_opaque_command_count: usize,
    pub(crate) transmission_command_count: usize,
    pub(crate) transparent_command_count: usize,
    pub(crate) velocity_command_count: usize,
    pub(crate) taa_reactive_mask_command_count: usize,
    pub(crate) direct_indexed_count: usize,
    pub(crate) indirect_indexed_count: usize,
    pub(crate) gpu_scene_instance_count: usize,
    pub(crate) cached_command_hit_count: usize,
    pub(crate) command_rebuild_count: usize,
    pub(crate) dynamic_command_count: usize,
    pub(crate) cache_miss_count: usize,
    pub(crate) cache_invalidated_transform_count: usize,
    pub(crate) cache_invalidated_geometry_count: usize,
    pub(crate) cache_invalidated_material_count: usize,
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
    pub(crate) fn from_cached_command_hits(
        commands: MeshDrawCommandList,
        cache_stats: MeshDrawCommandCacheStats,
    ) -> Self {
        Self::from_command_list(commands, cache_stats)
    }

    pub(crate) fn extend(&mut self, other: Self) {
        append_command_list(&mut self.depth_prepass, other.depth_prepass);
        append_command_list(&mut self.shadow, other.shadow);
        append_command_list(&mut self.opaque, other.opaque);
        append_command_list(&mut self.alpha_mask, other.alpha_mask);
        append_command_list(&mut self.advanced_pbr_opaque, other.advanced_pbr_opaque);
        append_command_list(&mut self.transmission, other.transmission);
        append_command_list(&mut self.transparent, other.transparent);
        append_command_list(&mut self.velocity, other.velocity);
        append_command_list(&mut self.taa_reactive_mask, other.taa_reactive_mask);
        self.cache_stats.accumulate(other.cache_stats);
    }

    fn from_command_list(
        commands: MeshDrawCommandList,
        cache_stats: MeshDrawCommandCacheStats,
    ) -> Self {
        let mut depth_prepass = Vec::new();
        let mut shadow = Vec::new();
        let mut opaque = Vec::new();
        let mut alpha_mask = Vec::new();
        let mut advanced_pbr_opaque = Vec::new();
        let mut transmission = Vec::new();
        let mut transparent = Vec::new();
        let mut velocity = Vec::new();
        let mut taa_reactive_mask = Vec::new();

        for command in commands.into_commands() {
            match command.phase {
                RenderPhase::Prepass => depth_prepass.push(command),
                RenderPhase::Shadow => shadow.push(command),
                RenderPhase::Opaque3d => opaque.push(command),
                RenderPhase::AlphaMask3d => alpha_mask.push(command),
                RenderPhase::Transparent3d if is_late_forward_opaque(&command) => {
                    advanced_pbr_opaque.push(command)
                }
                RenderPhase::Transparent3d if is_transmission(&command) => {
                    transmission.push(command)
                }
                RenderPhase::Transparent3d => transparent.push(command),
                RenderPhase::PostProcess
                    if command.pipeline_kind == super::MeshPassPipelineKind::Velocity =>
                {
                    velocity.push(command)
                }
                RenderPhase::PostProcess
                    if matches!(
                        command.pipeline_kind,
                        super::MeshPassPipelineKind::TaaReactiveMask
                            | super::MeshPassPipelineKind::TaaReactiveMaterialMask
                    ) =>
                {
                    taa_reactive_mask.push(command)
                }
                _ => {}
            }
        }

        Self {
            depth_prepass: MeshDrawCommandList::from_commands(depth_prepass),
            shadow: MeshDrawCommandList::from_commands(shadow),
            opaque: MeshDrawCommandList::from_commands(opaque),
            alpha_mask: MeshDrawCommandList::from_commands(alpha_mask),
            advanced_pbr_opaque: MeshDrawCommandList::from_commands(advanced_pbr_opaque),
            transmission: MeshDrawCommandList::from_commands(transmission),
            transparent: MeshDrawCommandList::from_commands(transparent),
            velocity: MeshDrawCommandList::from_commands(velocity),
            taa_reactive_mask: MeshDrawCommandList::from_commands(taa_reactive_mask),
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

    pub(crate) fn advanced_pbr_opaque(&self) -> &MeshDrawCommandList {
        &self.advanced_pbr_opaque
    }

    pub(crate) fn transmission(&self) -> &MeshDrawCommandList {
        &self.transmission
    }

    pub(crate) fn transparent(&self) -> &MeshDrawCommandList {
        &self.transparent
    }

    pub(crate) fn velocity(&self) -> &MeshDrawCommandList {
        &self.velocity
    }

    pub(crate) fn taa_reactive_mask(&self) -> &MeshDrawCommandList {
        &self.taa_reactive_mask
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
        let advanced_pbr_opaque = self.advanced_pbr_opaque.stats();
        let transmission = self.transmission.stats();
        let transparent = self.transparent.stats();
        let velocity = self.velocity.stats();
        let taa_reactive_mask = self.taa_reactive_mask.stats();
        let lists = [
            depth_prepass,
            shadow,
            opaque,
            alpha_mask,
            advanced_pbr_opaque,
            transmission,
            transparent,
            velocity,
            taa_reactive_mask,
        ];

        let mut indirect_stats = indirect_batch_stats(
            capabilities,
            [
                self.depth_prepass.commands(),
                self.shadow.commands(),
                self.opaque.commands(),
                self.alpha_mask.commands(),
                self.advanced_pbr_opaque.commands(),
                self.transparent.commands(),
                self.velocity.commands(),
                self.taa_reactive_mask.commands(),
            ],
        );
        accumulate_indirect_batch_stats(
            &mut indirect_stats,
            IndirectDrawBatcher::build(
                self.transmission.commands(),
                &RenderCapabilitySummary::default(),
            )
            .stats(),
        );

        MeshPassCommandBufferStats {
            command_count: lists.iter().map(|stats| stats.command_count).sum(),
            depth_prepass_command_count: depth_prepass.command_count,
            shadow_command_count: shadow.command_count,
            opaque_command_count: opaque.command_count,
            alpha_mask_command_count: alpha_mask.command_count,
            advanced_pbr_opaque_command_count: advanced_pbr_opaque.command_count,
            transmission_command_count: transmission.command_count,
            transparent_command_count: transparent.command_count,
            velocity_command_count: velocity.command_count,
            taa_reactive_mask_command_count: taa_reactive_mask.command_count,
            direct_indexed_count: lists.iter().map(|stats| stats.direct_indexed_count).sum(),
            indirect_indexed_count: lists.iter().map(|stats| stats.indirect_indexed_count).sum(),
            gpu_scene_instance_count: lists
                .iter()
                .map(|stats| stats.gpu_scene_instance_count)
                .sum(),
            cached_command_hit_count: self.cache_stats.cached_command_hit_count,
            command_rebuild_count: self.cache_stats.command_rebuild_count,
            dynamic_command_count: self.cache_stats.dynamic_command_count,
            cache_miss_count: self.cache_stats.cache_miss_count,
            cache_invalidated_transform_count: self.cache_stats.cache_invalidated_transform_count,
            cache_invalidated_geometry_count: self.cache_stats.cache_invalidated_geometry_count,
            cache_invalidated_material_count: self.cache_stats.cache_invalidated_material_count,
            ..indirect_stats
        }
    }
}

fn is_late_forward_opaque(command: &MeshDrawCommand) -> bool {
    command.pipeline_key().requires_forward_path() && !command.pipeline_key().pbr_transmission
}

fn is_transmission(command: &MeshDrawCommand) -> bool {
    command.pipeline_key().pbr_transmission
}

fn append_command_list(target: &mut MeshDrawCommandList, source: MeshDrawCommandList) {
    target.commands.extend(source.into_commands());
    target.sort();
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
