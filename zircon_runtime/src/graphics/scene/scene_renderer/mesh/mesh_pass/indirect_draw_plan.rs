use crate::core::framework::render::RenderCapabilitySummary;

use super::{
    IndirectCompactionBatchRange, IndirectCompactionPlan, IndirectDrawBatcher,
    IndirectDrawBatcherStats, MeshDrawCommand, MeshPassCommandBuffers,
};

pub(crate) struct MeshIndirectDrawPlan {
    pub(super) batcher: IndirectDrawBatcher,
    pub(super) compaction_plan: IndirectCompactionPlan,
}

#[derive(Default)]
pub(crate) struct MeshPassIndirectDrawPlans {
    pub(super) depth_prepass: Option<MeshIndirectDrawPlan>,
    pub(super) shadow: Option<MeshIndirectDrawPlan>,
    pub(super) opaque: Option<MeshIndirectDrawPlan>,
    pub(super) alpha_mask: Option<MeshIndirectDrawPlan>,
    pub(super) advanced_pbr_opaque: Option<MeshIndirectDrawPlan>,
    pub(super) transparent: Option<MeshIndirectDrawPlan>,
    pub(super) half_resolution_transparent: Option<MeshIndirectDrawPlan>,
    pub(super) velocity: Option<MeshIndirectDrawPlan>,
    pub(super) taa_reactive_mask: Option<MeshIndirectDrawPlan>,
    stats: IndirectDrawBatcherStats,
}

impl MeshIndirectDrawPlan {
    pub(crate) fn build(
        commands: &[MeshDrawCommand],
        capabilities: &RenderCapabilitySummary,
    ) -> (Option<Self>, IndirectDrawBatcherStats) {
        let batcher = IndirectDrawBatcher::build(commands, capabilities);
        let stats = batcher.stats();
        if batcher.args_cpu().is_empty() || batcher.batches().is_empty() {
            return (None, stats);
        }

        let batch_ranges = batcher.batches().iter().map(|batch| {
            IndirectCompactionBatchRange::new(
                batch.first_args,
                batch.args_count,
                batch.draw_count_index,
            )
        });
        let Some(compaction_plan) =
            IndirectCompactionPlan::try_from_ordered_batch_ranges(batcher.args_cpu(), batch_ranges)
        else {
            return (
                None,
                IndirectDrawBatcherStats {
                    fallback_draw_count: commands.len(),
                    ..IndirectDrawBatcherStats::default()
                },
            );
        };

        (
            Some(Self {
                batcher,
                compaction_plan,
            }),
            stats,
        )
    }
}

impl MeshPassIndirectDrawPlans {
    pub(crate) fn build(
        command_buffers: &MeshPassCommandBuffers,
        capabilities: &RenderCapabilitySummary,
    ) -> Self {
        let mut stats = IndirectDrawBatcherStats::default();
        let depth_prepass = build_phase_plan(
            command_buffers.depth_prepass().commands(),
            capabilities,
            &mut stats,
        );
        let shadow = build_phase_plan(
            command_buffers.shadow().commands(),
            capabilities,
            &mut stats,
        );
        let opaque = build_phase_plan(
            command_buffers.opaque().commands(),
            capabilities,
            &mut stats,
        );
        let alpha_mask = build_phase_plan(
            command_buffers.alpha_mask().commands(),
            capabilities,
            &mut stats,
        );
        let advanced_pbr_opaque = build_phase_plan(
            command_buffers.advanced_pbr_opaque().commands(),
            capabilities,
            &mut stats,
        );
        let transparent = build_phase_plan(
            command_buffers.transparent().commands(),
            capabilities,
            &mut stats,
        );
        let half_resolution_transparent = build_phase_plan(
            command_buffers.half_resolution_transparent().commands(),
            capabilities,
            &mut stats,
        );
        let velocity = build_phase_plan(
            command_buffers.velocity().commands(),
            capabilities,
            &mut stats,
        );
        let taa_reactive_mask = build_phase_plan(
            command_buffers.taa_reactive_mask().commands(),
            capabilities,
            &mut stats,
        );
        stats.fallback_draw_count = stats
            .fallback_draw_count
            .saturating_add(command_buffers.transmission().commands().len());

        Self {
            depth_prepass,
            shadow,
            opaque,
            alpha_mask,
            advanced_pbr_opaque,
            transparent,
            half_resolution_transparent,
            velocity,
            taa_reactive_mask,
            stats,
        }
    }

    pub(crate) const fn stats(&self) -> IndirectDrawBatcherStats {
        self.stats
    }
}

fn build_phase_plan(
    commands: &[MeshDrawCommand],
    capabilities: &RenderCapabilitySummary,
    stats: &mut IndirectDrawBatcherStats,
) -> Option<MeshIndirectDrawPlan> {
    let (plan, phase_stats) = MeshIndirectDrawPlan::build(commands, capabilities);
    accumulate_stats(stats, phase_stats);
    plan
}

fn accumulate_stats(total: &mut IndirectDrawBatcherStats, phase: IndirectDrawBatcherStats) {
    total.batch_count = total.batch_count.saturating_add(phase.batch_count);
    total.batched_draw_count = total
        .batched_draw_count
        .saturating_add(phase.batched_draw_count);
    total.fallback_draw_count = total
        .fallback_draw_count
        .saturating_add(phase.fallback_draw_count);
    total.indirect_args_count = total
        .indirect_args_count
        .saturating_add(phase.indirect_args_count);
}

#[cfg(test)]
mod tests;
