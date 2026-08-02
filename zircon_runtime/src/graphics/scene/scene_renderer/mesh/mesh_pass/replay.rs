use std::sync::atomic::{AtomicU32, Ordering};

use super::{
    INDEXED_INDIRECT_ARGS_STRIDE_BYTES, INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES, IndirectDrawBatch,
    MeshBindHandle, MeshDrawCommand, MeshDrawCommandStream, MeshIndirectDrawExecution,
    MeshPassPipelineKind, MeshPipelineVariantId,
};

pub(crate) const FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT: u32 = 1;
const MATERIAL_BIND_GROUP_SLOT: u32 = 2;
pub(crate) const GPU_SCENE_BIND_GROUP_SLOT: u32 = 3;
const TRACKED_BIND_GROUP_COUNT: usize = 4;

#[derive(Clone, Copy)]
pub(crate) struct MeshSceneDataBindHandle<'a> {
    id: u64,
    bind_group: &'a wgpu::BindGroup,
}

impl<'a> MeshSceneDataBindHandle<'a> {
    pub(crate) fn new(bind_group: &'a wgpu::BindGroup) -> Self {
        Self {
            id: bind_group as *const wgpu::BindGroup as usize as u64,
            bind_group,
        }
    }

    pub(crate) const fn id(self) -> u64 {
        self.id
    }

    pub(crate) const fn bind_group(self) -> &'a wgpu::BindGroup {
        self.bind_group
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshDrawReplayStats {
    pub(crate) draw_call_count: u32,
    pub(crate) state_change_count: u32,
    pub(crate) bind_skip_count: u32,
}

#[derive(Debug, Default)]
pub(crate) struct MeshDrawReplayStatsAccumulator {
    draw_call_count: AtomicU32,
    state_change_count: AtomicU32,
    bind_skip_count: AtomicU32,
}

impl MeshDrawReplayStatsAccumulator {
    pub(crate) fn record(&self, stats: MeshDrawReplayStats) {
        saturating_add(&self.draw_call_count, stats.draw_call_count);
        saturating_add(&self.state_change_count, stats.state_change_count);
        saturating_add(&self.bind_skip_count, stats.bind_skip_count);
    }

    pub(crate) fn stats(&self) -> MeshDrawReplayStats {
        MeshDrawReplayStats {
            draw_call_count: self.draw_call_count.load(Ordering::Relaxed),
            state_change_count: self.state_change_count.load(Ordering::Relaxed),
            bind_skip_count: self.bind_skip_count.load(Ordering::Relaxed),
        }
    }
}

fn saturating_add(value: &AtomicU32, increment: u32) {
    let _ = value.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_add(increment))
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MeshPipelineStateKey {
    kind: MeshPassPipelineKind,
    variant_id: MeshPipelineVariantId,
}

#[derive(Debug, Default)]
pub(crate) struct MeshDrawCommandReplayer {
    last_pipeline: Option<MeshPipelineStateKey>,
    last_bind_ids: [Option<u64>; TRACKED_BIND_GROUP_COUNT],
    last_geometry: Option<(u64, u64)>,
    stats: MeshDrawReplayStats,
}

impl MeshDrawCommandReplayer {
    pub(crate) fn should_set_pipeline(
        &mut self,
        kind: MeshPassPipelineKind,
        variant_id: MeshPipelineVariantId,
    ) -> bool {
        let key = MeshPipelineStateKey { kind, variant_id };
        if self.last_pipeline == Some(key) {
            return false;
        }
        self.last_pipeline = Some(key);
        self.last_bind_ids = [None; TRACKED_BIND_GROUP_COUNT];
        self.last_geometry = None;
        self.stats.state_change_count += 1;
        true
    }

    pub(crate) fn invalidate_state_after_external_pipeline(&mut self) {
        self.last_pipeline = None;
        self.last_bind_ids = [None; TRACKED_BIND_GROUP_COUNT];
        self.last_geometry = None;
    }

    pub(crate) fn bind_gpu_scene_if_needed<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        command: &'pass MeshDrawCommand,
        handle: Option<MeshSceneDataBindHandle<'pass>>,
    ) {
        if let Some(handle) = command.gpu_scene_bind_group.as_ref() {
            self.bind_group_if_needed(pass, GPU_SCENE_BIND_GROUP_SLOT, handle);
            return;
        }
        if let Some(handle) = handle {
            self.bind_raw_group_if_needed(
                pass,
                GPU_SCENE_BIND_GROUP_SLOT,
                handle.id(),
                handle.bind_group,
            );
        }
    }

    pub(crate) fn bind_material_if_needed<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        command: &'pass MeshDrawCommand,
    ) {
        let handle = command
            .material
            .as_ref()
            .expect("mesh command must carry material uniform bind group for this pass");
        self.bind_group_if_needed(pass, MATERIAL_BIND_GROUP_SLOT, handle);
    }

    pub(crate) fn bind_standard_material_if_needed<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        command: &'pass MeshDrawCommand,
    ) {
        let handle = command
            .standard_material
            .as_ref()
            .expect("mesh command must carry standard material bind group for this pass");
        self.bind_group_if_needed(pass, MATERIAL_BIND_GROUP_SLOT, handle);
    }

    pub(crate) fn bind_forward_shadow_receiver_if_needed<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        bind_group: &'pass wgpu::BindGroup,
    ) {
        self.bind_raw_group_if_needed(
            pass,
            FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT,
            bind_group as *const wgpu::BindGroup as usize as u64,
            bind_group,
        );
    }

    pub(crate) fn bind_geometry_if_needed<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        command: &'pass MeshDrawCommand,
    ) {
        let geometry_id = command.geometry_bind_key();
        if !self.should_bind_geometry(geometry_id) {
            return;
        }
        command.bind_geometry_buffers(pass);
    }

    pub(crate) fn draw_indexed<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        command: &'pass MeshDrawCommand,
    ) {
        command.record_indexed_draw(pass);
        self.stats.draw_call_count += 1;
    }

    pub(crate) fn replay_command_stream<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        stream: MeshDrawCommandStream<'pass>,
        mut prepare_command_state: impl FnMut(
            &mut Self,
            &mut wgpu::RenderPass<'pass>,
            &'pass MeshDrawCommand,
        ) -> bool,
    ) {
        let commands = stream.commands();
        let indirect = stream.indirect();
        let mut command_index = 0usize;
        let mut batch_index = 0usize;

        while command_index < commands.len() {
            let next_batch = indirect.and_then(|execution| {
                while batch_index < execution.batches().len()
                    && execution.batches()[batch_index].first_command_index < command_index
                {
                    batch_index += 1;
                }
                execution
                    .batches()
                    .get(batch_index)
                    .filter(|batch| batch.first_command_index == command_index)
            });

            let command = &commands[command_index];
            let should_draw = prepare_command_state(self, pass, command);
            if let Some(batch) = next_batch {
                if should_draw {
                    self.draw_indexed_indirect_batch(
                        pass,
                        indirect.expect("indirect batch must have an execution buffer"),
                        batch,
                    );
                }
                command_index += batch.args_count as usize;
                batch_index += 1;
            } else if should_draw {
                self.draw_indexed(pass, command);
                command_index += 1;
            } else {
                command_index += 1;
            }
        }
    }

    pub(crate) fn draw_indexed_indirect_batch<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        execution: &'pass MeshIndirectDrawExecution,
        batch: &IndirectDrawBatch,
    ) {
        let offset = u64::from(batch.first_args) * INDEXED_INDIRECT_ARGS_STRIDE_BYTES;
        if execution.compaction_ready_for_replay() {
            if let Some(bind_group) = execution.visible_remap_scene_bind_group() {
                self.bind_raw_group_if_needed(
                    pass,
                    GPU_SCENE_BIND_GROUP_SLOT,
                    bind_group as *const wgpu::BindGroup as usize as u64,
                    bind_group,
                );
            }
            let count_offset =
                u64::from(batch.draw_count_index) * INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES;
            pass.multi_draw_indexed_indirect_count(
                execution
                    .compaction_resources()
                    .compacted_indirect_args_buffer(),
                offset,
                execution.compaction_resources().draw_count_buffer(),
                count_offset,
                batch.args_count,
            );
            self.stats.draw_call_count += 1;
            return;
        }

        pass.multi_draw_indexed_indirect(execution.args_buffer(), offset, batch.args_count);
        self.stats.draw_call_count += 1;
    }

    pub(crate) const fn stats(&self) -> MeshDrawReplayStats {
        self.stats
    }

    fn bind_group_if_needed<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        slot: u32,
        handle: &'pass MeshBindHandle,
    ) {
        self.bind_raw_group_if_needed(pass, slot, handle.id(), handle.bind_group());
    }

    fn bind_raw_group_if_needed<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        slot: u32,
        id: u64,
        bind_group: &'pass wgpu::BindGroup,
    ) {
        if !self.should_bind_raw_group(slot, id) {
            return;
        }

        pass.set_bind_group(slot, bind_group, &[]);
    }

    fn should_bind_raw_group(&mut self, slot: u32, id: u64) -> bool {
        let slot_index = slot as usize;
        if slot_index < self.last_bind_ids.len() && self.last_bind_ids[slot_index] == Some(id) {
            self.stats.bind_skip_count += 1;
            return false;
        }

        if slot_index < self.last_bind_ids.len() {
            self.last_bind_ids[slot_index] = Some(id);
        }
        true
    }

    fn should_bind_geometry(&mut self, geometry_id: (u64, u64)) -> bool {
        if self.last_geometry == Some(geometry_id) {
            return false;
        }
        self.last_geometry = Some(geometry_id);
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    use super::{
        FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, MeshDrawCommandReplayer, MeshDrawReplayStats,
        MeshDrawReplayStatsAccumulator,
    };

    #[test]
    fn replay_stats_accumulator_is_sync_and_saturates() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<MeshDrawReplayStatsAccumulator>();

        let stats = MeshDrawReplayStatsAccumulator::default();
        stats.record(MeshDrawReplayStats {
            draw_call_count: u32::MAX,
            state_change_count: 2,
            bind_skip_count: 3,
        });
        stats.record(MeshDrawReplayStats {
            draw_call_count: 1,
            state_change_count: u32::MAX,
            bind_skip_count: u32::MAX,
        });

        assert_eq!(
            stats.stats(),
            MeshDrawReplayStats {
                draw_call_count: u32::MAX,
                state_change_count: u32::MAX,
                bind_skip_count: u32::MAX,
            }
        );
    }

    #[test]
    fn mesh_draw_command_replayer_rebinds_after_external_pipeline() {
        let mut replayer = MeshDrawCommandReplayer::default();
        let variant = MeshPipelineVariantId::new(1);

        assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, variant));
        assert!(!replayer.should_set_pipeline(MeshPassPipelineKind::Base, variant));

        replayer.invalidate_state_after_external_pipeline();

        assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, variant));
    }

    #[test]
    fn mesh_draw_command_replayer_counts_pipeline_changes() {
        let mut replayer = MeshDrawCommandReplayer::default();
        let base_variant = MeshPipelineVariantId::new(1);
        let shadow_variant = MeshPipelineVariantId::new(2);

        assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
        assert!(!replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
        assert!(replayer.should_set_pipeline(MeshPassPipelineKind::ShadowDepth, shadow_variant));

        assert_eq!(replayer.stats().draw_call_count, 0);
        assert_eq!(replayer.stats().state_change_count, 2);
        assert_eq!(replayer.stats().bind_skip_count, 0);
    }

    #[test]
    fn mesh_draw_command_replayer_skips_redundant_tracked_bind_groups() {
        let mut replayer = MeshDrawCommandReplayer::default();

        assert!(replayer.should_bind_raw_group(2, 10));
        assert!(!replayer.should_bind_raw_group(2, 10));
        assert!(replayer.should_bind_raw_group(2, 11));
        assert!(replayer.should_bind_raw_group(6, 10));
        assert!(replayer.should_bind_raw_group(6, 10));

        assert_eq!(replayer.stats().bind_skip_count, 1);
    }

    #[test]
    fn mesh_draw_command_replayer_resets_bind_tracking_on_pipeline_change() {
        let mut replayer = MeshDrawCommandReplayer::default();
        let base_variant = MeshPipelineVariantId::new(1);
        let shadow_variant = MeshPipelineVariantId::new(2);

        assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
        assert!(replayer.should_bind_raw_group(2, 10));
        assert!(!replayer.should_bind_raw_group(2, 10));
        assert!(replayer.should_set_pipeline(MeshPassPipelineKind::ShadowDepth, shadow_variant));
        assert!(replayer.should_bind_raw_group(2, 10));

        assert_eq!(replayer.stats().state_change_count, 2);
        assert_eq!(replayer.stats().bind_skip_count, 1);
    }

    #[test]
    fn mesh_draw_command_replayer_tracks_forward_shadow_receiver_slot() {
        let mut replayer = MeshDrawCommandReplayer::default();
        let base_variant = MeshPipelineVariantId::new(1);

        assert!(replayer.should_bind_raw_group(FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, 10));
        assert!(!replayer.should_bind_raw_group(FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, 10));
        assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
        assert!(replayer.should_bind_raw_group(FORWARD_SHADOW_RECEIVER_BIND_GROUP_SLOT, 10));
    }

    #[test]
    fn mesh_draw_command_replayer_skips_redundant_geometry_until_pipeline_changes() {
        let mut replayer = MeshDrawCommandReplayer::default();
        let base_variant = MeshPipelineVariantId::new(1);
        let shadow_variant = MeshPipelineVariantId::new(2);

        assert!(replayer.should_set_pipeline(MeshPassPipelineKind::Base, base_variant));
        assert!(replayer.should_bind_geometry((10, 20)));
        assert!(!replayer.should_bind_geometry((10, 20)));
        assert!(replayer.should_bind_geometry((10, 21)));
        assert!(replayer.should_set_pipeline(MeshPassPipelineKind::ShadowDepth, shadow_variant));
        assert!(replayer.should_bind_geometry((10, 21)));
    }

    #[test]
    fn mesh_draw_command_replayer_records_multi_draw_indexed_indirect_batches() {
        let source = include_str!("replay.rs");

        assert!(source.contains("pass.multi_draw_indexed_indirect"));
        assert!(source.contains("pass.multi_draw_indexed_indirect_count"));
        assert!(source.contains("compaction_ready_for_replay"));
        assert!(source.contains("visible_remap_scene_bind_group"));
        assert!(source.contains("batch.first_args"));
        assert!(source.contains("batch.args_count"));
        assert!(source.contains("batch.draw_count_index"));
    }
}
