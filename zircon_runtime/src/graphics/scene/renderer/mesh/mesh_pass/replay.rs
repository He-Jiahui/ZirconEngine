use std::sync::atomic::{AtomicU32, Ordering};

use super::{
    IndirectDrawBatch, MeshBindHandle, MeshDrawArgs, MeshDrawCommand, MeshDrawCommandStream,
    MeshIndirectDrawExecution, MeshPassPipelineKind, MeshPipelineVariantId,
    INDEXED_INDIRECT_ARGS_STRIDE_BYTES, INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
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
    pub(crate) indirect_count_draw_call_count: u32,
    pub(crate) fixed_multi_draw_call_count: u32,
    pub(crate) per_draw_indirect_draw_call_count: u32,
    pub(crate) direct_draw_call_count: u32,
    pub(crate) state_change_count: u32,
    pub(crate) bind_skip_count: u32,
    pub(crate) material_bind_group_set_count: u32,
    pub(crate) material_bind_group_skip_count: u32,
}

#[derive(Debug, Default)]
pub(crate) struct MeshDrawReplayStatsAccumulator {
    draw_call_count: AtomicU32,
    indirect_count_draw_call_count: AtomicU32,
    fixed_multi_draw_call_count: AtomicU32,
    per_draw_indirect_draw_call_count: AtomicU32,
    direct_draw_call_count: AtomicU32,
    state_change_count: AtomicU32,
    bind_skip_count: AtomicU32,
    material_bind_group_set_count: AtomicU32,
    material_bind_group_skip_count: AtomicU32,
}

impl MeshDrawReplayStatsAccumulator {
    pub(crate) fn record(&self, stats: MeshDrawReplayStats) {
        saturating_add(&self.draw_call_count, stats.draw_call_count);
        saturating_add(
            &self.indirect_count_draw_call_count,
            stats.indirect_count_draw_call_count,
        );
        saturating_add(
            &self.fixed_multi_draw_call_count,
            stats.fixed_multi_draw_call_count,
        );
        saturating_add(
            &self.per_draw_indirect_draw_call_count,
            stats.per_draw_indirect_draw_call_count,
        );
        saturating_add(&self.direct_draw_call_count, stats.direct_draw_call_count);
        saturating_add(&self.state_change_count, stats.state_change_count);
        saturating_add(&self.bind_skip_count, stats.bind_skip_count);
        saturating_add(
            &self.material_bind_group_set_count,
            stats.material_bind_group_set_count,
        );
        saturating_add(
            &self.material_bind_group_skip_count,
            stats.material_bind_group_skip_count,
        );
    }

    pub(crate) fn stats(&self) -> MeshDrawReplayStats {
        MeshDrawReplayStats {
            draw_call_count: self.draw_call_count.load(Ordering::Relaxed),
            indirect_count_draw_call_count: self
                .indirect_count_draw_call_count
                .load(Ordering::Relaxed),
            fixed_multi_draw_call_count: self.fixed_multi_draw_call_count.load(Ordering::Relaxed),
            per_draw_indirect_draw_call_count: self
                .per_draw_indirect_draw_call_count
                .load(Ordering::Relaxed),
            direct_draw_call_count: self.direct_draw_call_count.load(Ordering::Relaxed),
            state_change_count: self.state_change_count.load(Ordering::Relaxed),
            bind_skip_count: self.bind_skip_count.load(Ordering::Relaxed),
            material_bind_group_set_count: self
                .material_bind_group_set_count
                .load(Ordering::Relaxed),
            material_bind_group_skip_count: self
                .material_bind_group_skip_count
                .load(Ordering::Relaxed),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrackedBindGroupIdentity {
    Owned(u64),
    Borrowed(u64),
}

#[derive(Debug, Default)]
pub(crate) struct MeshDrawCommandReplayer {
    last_pipeline: Option<MeshPipelineStateKey>,
    last_bind_ids: [Option<TrackedBindGroupIdentity>; TRACKED_BIND_GROUP_COUNT],
    last_geometry: Option<(u64, u64)>,
    stats: MeshDrawReplayStats,
}

impl MeshDrawCommandReplayer {
    /// 此判断立即更新回放缓存；若管线准入延后或失败，调用方须 invalidate，避免同 key 被误认为已经绑定。
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
        self.stats.state_change_count = self.stats.state_change_count.saturating_add(1);
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
        self.record_unbatched_draw_call(&command.draw_args);
    }

    fn record_unbatched_draw_call(&mut self, draw_args: &MeshDrawArgs) {
        self.stats.draw_call_count = self.stats.draw_call_count.saturating_add(1);
        if draw_args.is_indirect() {
            self.stats.per_draw_indirect_draw_call_count = self
                .stats
                .per_draw_indirect_draw_call_count
                .saturating_add(1);
        } else {
            self.stats.direct_draw_call_count = self.stats.direct_draw_call_count.saturating_add(1);
        }
    }

    /// 间接批次仅为首条命令调用准备回调，绘制决定应用到整批；需要逐实体过滤时，调用方须先移除 indirect 执行信息。
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
        crate::profile_scope!("render", "mesh_commands", "replay_record");
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
            if execution.indirect_count_supported() {
                pass.multi_draw_indexed_indirect_count(
                    execution
                        .compaction_resources()
                        .compacted_indirect_args_buffer(),
                    offset,
                    execution.compaction_resources().draw_count_buffer(),
                    count_offset,
                    batch.args_count,
                );
                self.record_indirect_count_draw_call();
            } else if execution.multi_draw_indirect_supported() {
                // Compaction clears the full args range before writing its dense prefix, so the
                // fixed-count fallback safely consumes zero-instance tail entries.
                pass.multi_draw_indexed_indirect(
                    execution
                        .compaction_resources()
                        .compacted_indirect_args_buffer(),
                    offset,
                    batch.args_count,
                );
                self.record_fixed_multi_draw_call();
            } else {
                self.draw_indexed_indirect_range(
                    pass,
                    execution
                        .compaction_resources()
                        .compacted_indirect_args_buffer(),
                    offset,
                    batch.args_count,
                );
            }
            return;
        }

        if execution.multi_draw_indirect_supported() {
            pass.multi_draw_indexed_indirect(execution.args_buffer(), offset, batch.args_count);
            self.record_fixed_multi_draw_call();
        } else {
            self.draw_indexed_indirect_range(
                pass,
                execution.args_buffer(),
                offset,
                batch.args_count,
            );
        }
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
        if !self.should_bind_mesh_group(slot, handle.id()) {
            return;
        }

        pass.set_bind_group(slot, handle.bind_group(), &[]);
    }

    fn draw_indexed_indirect_range<'pass>(
        &mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        args_buffer: &'pass wgpu::Buffer,
        first_offset: u64,
        args_count: u32,
    ) {
        for argument_index in 0..args_count {
            let offset = first_offset
                .saturating_add(u64::from(argument_index) * INDEXED_INDIRECT_ARGS_STRIDE_BYTES);
            pass.draw_indexed_indirect(args_buffer, offset);
        }
        self.stats.draw_call_count = self.stats.draw_call_count.saturating_add(args_count);
        self.stats.per_draw_indirect_draw_call_count = self
            .stats
            .per_draw_indirect_draw_call_count
            .saturating_add(args_count);
    }

    fn record_indirect_count_draw_call(&mut self) {
        self.stats.draw_call_count = self.stats.draw_call_count.saturating_add(1);
        self.stats.indirect_count_draw_call_count =
            self.stats.indirect_count_draw_call_count.saturating_add(1);
    }

    fn record_fixed_multi_draw_call(&mut self) {
        self.stats.draw_call_count = self.stats.draw_call_count.saturating_add(1);
        self.stats.fixed_multi_draw_call_count =
            self.stats.fixed_multi_draw_call_count.saturating_add(1);
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
        self.should_bind_group(slot, TrackedBindGroupIdentity::Borrowed(id))
    }

    fn should_bind_mesh_group(&mut self, slot: u32, id: u64) -> bool {
        self.should_bind_group(slot, TrackedBindGroupIdentity::Owned(id))
    }

    fn should_bind_group(&mut self, slot: u32, identity: TrackedBindGroupIdentity) -> bool {
        let slot_index = slot as usize;
        if slot_index < self.last_bind_ids.len() && self.last_bind_ids[slot_index] == Some(identity)
        {
            self.stats.bind_skip_count = self.stats.bind_skip_count.saturating_add(1);
            if slot == MATERIAL_BIND_GROUP_SLOT {
                self.stats.material_bind_group_skip_count =
                    self.stats.material_bind_group_skip_count.saturating_add(1);
            }
            return false;
        }

        if slot_index < self.last_bind_ids.len() {
            self.last_bind_ids[slot_index] = Some(identity);
        }
        if slot == MATERIAL_BIND_GROUP_SLOT {
            self.stats.material_bind_group_set_count =
                self.stats.material_bind_group_set_count.saturating_add(1);
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
#[path = "tests/replay.rs"]
mod tests;
