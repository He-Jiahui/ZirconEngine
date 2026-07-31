use std::{cell::Cell, sync::mpsc};

use wgpu::util::DeviceExt;

use crate::core::framework::render::RenderCapabilitySummary;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;

use super::{
    IndirectCompactionBatchRange, IndirectCompactionPlan, IndirectDrawBatch, IndirectDrawBatcher,
    MeshDrawCommand, MeshIndirectCompactionResources, MeshPassCommandBuffers,
};

pub(crate) const INDEXED_INDIRECT_ARGS_STRIDE_BYTES: wgpu::BufferAddress =
    std::mem::size_of::<IndexedIndirectArgs>() as wgpu::BufferAddress;
const DRAW_COUNT_STRIDE_BYTES: wgpu::BufferAddress =
    std::mem::size_of::<u32>() as wgpu::BufferAddress;

#[derive(Clone, Copy)]
pub(crate) struct MeshDrawCommandStream<'a> {
    commands: &'a [MeshDrawCommand],
    indirect: Option<&'a MeshIndirectDrawExecution>,
}

impl<'a> MeshDrawCommandStream<'a> {
    pub(crate) fn new(
        commands: &'a [MeshDrawCommand],
        indirect: Option<&'a MeshIndirectDrawExecution>,
    ) -> Self {
        Self { commands, indirect }
    }

    pub(crate) fn empty() -> Self {
        Self {
            commands: &[],
            indirect: None,
        }
    }

    pub(crate) fn commands(self) -> &'a [MeshDrawCommand] {
        self.commands
    }

    pub(crate) fn indirect(self) -> Option<&'a MeshIndirectDrawExecution> {
        self.indirect
    }

    pub(crate) fn is_empty(self) -> bool {
        self.commands.is_empty()
    }
}

pub(crate) struct MeshIndirectDrawExecution {
    args_buffer: wgpu::Buffer,
    batches: Vec<IndirectDrawBatch>,
    compaction_plan: IndirectCompactionPlan,
    compaction_resources: MeshIndirectCompactionResources,
    visible_remap_scene_bind_group: Option<wgpu::BindGroup>,
    compaction_ready_for_replay: Cell<bool>,
    args_count: u32,
    total_instances: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshIndirectArgsSnapshot {
    args: Vec<IndexedIndirectArgs>,
    draw_counts: Vec<u32>,
}

pub(crate) struct MeshIndirectArgsReadback {
    buffer: wgpu::Buffer,
    args_count: u32,
    draw_count_buffer: Option<wgpu::Buffer>,
    draw_count_count: u32,
}

impl MeshIndirectDrawExecution {
    pub(crate) fn build(
        device: &wgpu::Device,
        label: &'static str,
        commands: &[MeshDrawCommand],
        capabilities: &RenderCapabilitySummary,
    ) -> Option<Self> {
        let batcher = IndirectDrawBatcher::build(commands, capabilities);
        if batcher.args_cpu().is_empty() || batcher.batches().is_empty() {
            return None;
        }

        let batch_ranges = batcher.batches().iter().map(|batch| {
            IndirectCompactionBatchRange::new(
                batch.first_args,
                batch.args_count,
                batch.draw_count_index,
            )
        });
        let compaction_plan = IndirectCompactionPlan::try_from_args_and_batch_ranges(
            batcher.args_cpu(),
            batch_ranges,
        )?;
        let compaction_resources =
            MeshIndirectCompactionResources::new(device, label, &compaction_plan);
        let args_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(batcher.args_cpu()),
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });
        let args_count = batcher.args_cpu().len() as u32;
        let total_instances = batcher
            .batches()
            .iter()
            .map(|batch| batch.total_instances)
            .sum();

        Some(Self {
            args_buffer,
            batches: batcher.batches().to_vec(),
            compaction_plan,
            compaction_resources,
            visible_remap_scene_bind_group: None,
            compaction_ready_for_replay: Cell::new(false),
            args_count,
            total_instances,
        })
    }

    pub(crate) fn args_buffer(&self) -> &wgpu::Buffer {
        &self.args_buffer
    }

    pub(crate) fn replay_args_buffer(&self) -> &wgpu::Buffer {
        if self.compaction_ready_for_replay() {
            return self.compaction_resources.compacted_indirect_args_buffer();
        }
        &self.args_buffer
    }

    pub(crate) fn batches(&self) -> &[IndirectDrawBatch] {
        &self.batches
    }

    pub(crate) fn compaction_plan(&self) -> &IndirectCompactionPlan {
        &self.compaction_plan
    }

    pub(crate) fn compaction_resources(&self) -> &MeshIndirectCompactionResources {
        &self.compaction_resources
    }

    pub(crate) fn visible_remap_scene_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.visible_remap_scene_bind_group.as_ref()
    }

    pub(crate) fn attach_visible_remap_scene_bind_group(&mut self, bind_group: wgpu::BindGroup) {
        self.visible_remap_scene_bind_group = Some(bind_group);
    }

    pub(crate) fn mark_compaction_ready_for_replay(&self) {
        self.compaction_ready_for_replay.set(true);
    }

    pub(crate) fn compaction_ready_for_replay(&self) -> bool {
        self.compaction_ready_for_replay.get()
    }

    pub(crate) const fn args_count(&self) -> u32 {
        self.args_count
    }

    pub(crate) const fn total_instances(&self) -> u32 {
        self.total_instances
    }

    pub(crate) fn copy_args_to_readback(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        label: &'static str,
    ) -> MeshIndirectArgsReadback {
        let byte_size = self.args_readback_byte_size();
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: byte_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(self.replay_args_buffer(), 0, &readback, 0, byte_size);

        let draw_count_count = self.compaction_resources.draw_count_capacity();
        let draw_count_byte_size = self.compaction_resources.draw_count_buffer_byte_size();
        let draw_count_buffer = (self.compaction_ready_for_replay()
            && draw_count_count > 0
            && draw_count_byte_size > 0)
            .then(|| {
                let readback = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size: draw_count_byte_size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                });
                encoder.copy_buffer_to_buffer(
                    self.compaction_resources.draw_count_buffer(),
                    0,
                    &readback,
                    0,
                    draw_count_byte_size,
                );
                readback
            });
        let copied_draw_count_count = if draw_count_buffer.is_some() {
            draw_count_count.min((draw_count_byte_size / DRAW_COUNT_STRIDE_BYTES) as u32)
        } else {
            0
        };

        MeshIndirectArgsReadback {
            buffer: readback,
            args_count: self.args_count,
            draw_count_buffer,
            draw_count_count: copied_draw_count_count,
        }
    }

    fn args_readback_byte_size(&self) -> wgpu::BufferAddress {
        u64::from(self.args_count) * INDEXED_INDIRECT_ARGS_STRIDE_BYTES
    }
}

impl MeshIndirectArgsSnapshot {
    pub(crate) fn from_args(args: Vec<IndexedIndirectArgs>) -> Self {
        Self {
            args,
            draw_counts: Vec::new(),
        }
    }

    pub(crate) fn from_args_and_draw_counts(
        args: Vec<IndexedIndirectArgs>,
        draw_counts: Vec<u32>,
    ) -> Self {
        Self { args, draw_counts }
    }

    pub(crate) fn args_count(&self) -> u32 {
        self.args.len().min(u32::MAX as usize) as u32
    }

    pub(crate) fn compacted_draw_count(&self) -> u32 {
        self.draw_counts
            .iter()
            .fold(0, |count, draw_count| count.saturating_add(*draw_count))
    }

    pub(crate) fn zero_instance_arg_count(&self) -> u32 {
        self.args
            .iter()
            .filter(|args| args.instance_count == 0)
            .count()
            .min(u32::MAX as usize) as u32
    }

    pub(crate) fn remaining_instance_count(&self) -> u32 {
        self.args
            .iter()
            .fold(0, |count, args| count.saturating_add(args.instance_count))
    }
}

impl MeshIndirectArgsReadback {
    pub(crate) fn collect(self, device: &wgpu::Device) -> Option<MeshIndirectArgsSnapshot> {
        let byte_size = u64::from(self.args_count) * INDEXED_INDIRECT_ARGS_STRIDE_BYTES;
        let args = collect_pod_buffer::<IndexedIndirectArgs>(device, &self.buffer, byte_size)?;
        let draw_counts = if let Some(draw_count_buffer) = self.draw_count_buffer.as_ref() {
            let byte_size = u64::from(self.draw_count_count) * DRAW_COUNT_STRIDE_BYTES;
            collect_pod_buffer::<u32>(device, draw_count_buffer, byte_size)?
        } else {
            Vec::new()
        };

        Some(MeshIndirectArgsSnapshot::from_args_and_draw_counts(
            args,
            draw_counts,
        ))
    }
}

fn collect_pod_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    byte_size: wgpu::BufferAddress,
) -> Option<Vec<T>> {
    if byte_size == 0 {
        return Some(Vec::new());
    }

    let slice = buffer.slice(0..byte_size);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
    receiver.recv().ok()?.ok()?;

    let mapped = slice.get_mapped_range();
    let values = bytemuck::cast_slice::<u8, T>(&mapped[..byte_size as usize]).to_vec();
    drop(mapped);
    buffer.unmap();
    Some(values)
}

#[derive(Default)]
pub(crate) struct MeshPassIndirectDrawExecutions {
    depth_prepass: Option<MeshIndirectDrawExecution>,
    shadow: Option<MeshIndirectDrawExecution>,
    opaque: Option<MeshIndirectDrawExecution>,
    alpha_mask: Option<MeshIndirectDrawExecution>,
    advanced_pbr_opaque: Option<MeshIndirectDrawExecution>,
    transparent: Option<MeshIndirectDrawExecution>,
    velocity: Option<MeshIndirectDrawExecution>,
    taa_reactive_mask: Option<MeshIndirectDrawExecution>,
}

impl MeshPassIndirectDrawExecutions {
    pub(crate) fn build(
        device: &wgpu::Device,
        capabilities: &RenderCapabilitySummary,
        command_buffers: &MeshPassCommandBuffers,
    ) -> Self {
        Self {
            depth_prepass: MeshIndirectDrawExecution::build(
                device,
                "zircon-depth-prepass-indirect-args",
                command_buffers.depth_prepass().commands(),
                capabilities,
            ),
            shadow: MeshIndirectDrawExecution::build(
                device,
                "zircon-shadow-indirect-args",
                command_buffers.shadow().commands(),
                capabilities,
            ),
            opaque: MeshIndirectDrawExecution::build(
                device,
                "zircon-opaque-indirect-args",
                command_buffers.opaque().commands(),
                capabilities,
            ),
            alpha_mask: MeshIndirectDrawExecution::build(
                device,
                "zircon-alpha-mask-indirect-args",
                command_buffers.alpha_mask().commands(),
                capabilities,
            ),
            advanced_pbr_opaque: MeshIndirectDrawExecution::build(
                device,
                "zircon-advanced-pbr-opaque-indirect-args",
                command_buffers.advanced_pbr_opaque().commands(),
                capabilities,
            ),
            transparent: MeshIndirectDrawExecution::build(
                device,
                "zircon-transparent-indirect-args",
                command_buffers.transparent().commands(),
                capabilities,
            ),
            velocity: MeshIndirectDrawExecution::build(
                device,
                "zircon-velocity-indirect-args",
                command_buffers.velocity().commands(),
                capabilities,
            ),
            taa_reactive_mask: MeshIndirectDrawExecution::build(
                device,
                "zircon-taa-reactive-mask-indirect-args",
                command_buffers.taa_reactive_mask().commands(),
                capabilities,
            ),
        }
    }

    pub(crate) fn depth_prepass(&self) -> Option<&MeshIndirectDrawExecution> {
        self.depth_prepass.as_ref()
    }

    pub(crate) fn shadow(&self) -> Option<&MeshIndirectDrawExecution> {
        self.shadow.as_ref()
    }

    pub(crate) fn opaque(&self) -> Option<&MeshIndirectDrawExecution> {
        self.opaque.as_ref()
    }

    pub(crate) fn alpha_mask(&self) -> Option<&MeshIndirectDrawExecution> {
        self.alpha_mask.as_ref()
    }

    pub(crate) fn advanced_pbr_opaque(&self) -> Option<&MeshIndirectDrawExecution> {
        self.advanced_pbr_opaque.as_ref()
    }

    pub(crate) fn transparent(&self) -> Option<&MeshIndirectDrawExecution> {
        self.transparent.as_ref()
    }

    pub(crate) fn velocity(&self) -> Option<&MeshIndirectDrawExecution> {
        self.velocity.as_ref()
    }

    pub(crate) fn taa_reactive_mask(&self) -> Option<&MeshIndirectDrawExecution> {
        self.taa_reactive_mask.as_ref()
    }

    pub(crate) fn attach_visible_remap_scene_bind_groups(
        &mut self,
        device: &wgpu::Device,
        gpu_scene: &GpuScene,
    ) {
        for execution in self.executions_mut().into_iter().flatten() {
            let bind_group = gpu_scene.create_scene_bind_group_for_visible_instance_remap(
                device,
                execution
                    .compaction_resources()
                    .visible_instance_index_buffer(),
            );
            execution.attach_visible_remap_scene_bind_group(bind_group);
        }
    }

    pub(crate) fn occlusion_cull_candidate_arg_count(&self) -> u32 {
        self.hzb_occlusion_executions()
            .into_iter()
            .flatten()
            .map(|execution| execution.args_count())
            .sum()
    }

    pub(crate) fn occlusion_cull_candidate_instance_count(&self) -> u32 {
        self.hzb_occlusion_executions()
            .into_iter()
            .flatten()
            .map(|execution| execution.total_instances())
            .sum()
    }

    pub(crate) fn copy_args_to_readbacks(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        label: &'static str,
    ) -> Vec<MeshIndirectArgsReadback> {
        self.executions()
            .into_iter()
            .flatten()
            .map(|execution| execution.copy_args_to_readback(device, encoder, label))
            .collect()
    }

    pub(crate) fn copy_hzb_occlusion_args_to_readbacks(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        label: &'static str,
    ) -> Vec<MeshIndirectArgsReadback> {
        self.hzb_occlusion_executions()
            .into_iter()
            .flatten()
            .map(|execution| execution.copy_args_to_readback(device, encoder, label))
            .collect()
    }

    pub(crate) fn hzb_occlusion_executions(&self) -> [Option<&MeshIndirectDrawExecution>; 4] {
        [
            self.opaque(),
            self.alpha_mask(),
            self.advanced_pbr_opaque(),
            self.velocity(),
        ]
    }

    fn executions(&self) -> [Option<&MeshIndirectDrawExecution>; 8] {
        [
            self.depth_prepass(),
            self.shadow(),
            self.opaque(),
            self.alpha_mask(),
            self.advanced_pbr_opaque(),
            self.transparent(),
            self.velocity(),
            self.taa_reactive_mask(),
        ]
    }

    fn executions_mut(&mut self) -> [Option<&mut MeshIndirectDrawExecution>; 8] {
        [
            self.depth_prepass.as_mut(),
            self.shadow.as_mut(),
            self.opaque.as_mut(),
            self.alpha_mask.as_mut(),
            self.advanced_pbr_opaque.as_mut(),
            self.transparent.as_mut(),
            self.velocity.as_mut(),
            self.taa_reactive_mask.as_mut(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{INDEXED_INDIRECT_ARGS_STRIDE_BYTES, MeshIndirectArgsSnapshot};
    use crate::core::framework::render::{RenderCapabilitySummary, RenderPhase};
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    #[test]
    fn mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer() {
        let source = include_str!("indirect_draw_execution.rs");

        assert_eq!(INDEXED_INDIRECT_ARGS_STRIDE_BYTES, 20);
        assert!(source.contains("create_buffer_init"));
        assert!(source.contains("wgpu::BufferUsages::INDIRECT"));
        assert!(source.contains("wgpu::BufferUsages::STORAGE"));
        assert!(source.contains("bytemuck::cast_slice(batcher.args_cpu())"));
    }

    #[test]
    fn mesh_indirect_args_snapshot_counts_zeroed_instance_args() {
        let snapshot = MeshIndirectArgsSnapshot::from_args_and_draw_counts(
            vec![
                indirect_args(10, 4, 2),
                indirect_args(20, 0, 8),
                indirect_args(30, 6, 12),
                indirect_args(40, 0, 18),
            ],
            vec![1, 2],
        );

        assert_eq!(snapshot.args_count(), 4);
        assert_eq!(snapshot.compacted_draw_count(), 3);
        assert_eq!(snapshot.zero_instance_arg_count(), 2);
        assert_eq!(snapshot.remaining_instance_count(), 10);
    }

    #[test]
    fn mesh_indirect_draw_execution_sources_readback_from_indirect_args_buffer() {
        let source = include_str!("indirect_draw_execution.rs");

        assert!(source.contains("copy_buffer_to_buffer(self.replay_args_buffer()"));
        assert!(source.contains("self.compaction_resources.draw_count_buffer()"));
        assert!(source.contains("wgpu::BufferUsages::MAP_READ"));
        assert!(source.contains("collect_pod_buffer::<IndexedIndirectArgs>"));
        assert!(source.contains("collect_pod_buffer::<u32>"));
    }

    #[test]
    fn mesh_indirect_draw_execution_builds_compaction_plan_from_uploaded_args() {
        let Some(backend) = crate::graphics::backend::RenderBackend::new_offscreen().ok() else {
            return;
        };
        let commands = vec![command(10, 1, 2, 3), command(20, 4, 8, 2)];

        let execution = super::MeshIndirectDrawExecution::build(
            &backend.device,
            "zircon-test-indirect-compaction-execution",
            &commands,
            &gpu_driven_capabilities(),
        )
        .expect("indirect execution");

        let plan = execution.compaction_plan();
        assert_eq!(plan.metadata_count(), 2);
        assert_eq!(plan.visible_instance_capacity(), 5);
        assert_eq!(plan.metadata()[0].visible_instance_base, 0);
        assert_eq!(plan.metadata()[0].source_first_instance, 2);
        assert_eq!(plan.metadata()[0].source_instance_count, 3);
        assert_eq!(plan.metadata()[1].visible_instance_base, 3);
        assert_eq!(plan.metadata()[1].source_first_instance, 8);
        assert_eq!(plan.metadata()[1].source_instance_count, 2);

        let resources = execution.compaction_resources();
        assert_eq!(
            resources.metadata_buffer_byte_size(),
            plan.metadata_buffer_byte_size()
        );
        assert_eq!(
            resources.visible_instance_index_buffer_byte_size(),
            plan.visible_instance_index_buffer_byte_size()
        );
        assert_eq!(resources.visible_instance_index_capacity(), 5);
        assert_eq!(
            resources.visible_instance_index_buffer_allocation_byte_size(),
            20
        );
        assert_eq!(resources.compacted_indirect_args_buffer_byte_size(), 40);
        assert_eq!(resources.draw_count_buffer_byte_size(), 4);
        assert_eq!(resources.draw_count_capacity(), 1);
    }

    fn indirect_args(
        index_count: u32,
        instance_count: u32,
        first_instance: u32,
    ) -> IndexedIndirectArgs {
        IndexedIndirectArgs {
            index_count,
            instance_count,
            first_index: 0,
            base_vertex: 0,
            first_instance,
        }
    }

    fn command(
        index_count: u32,
        first_index: u32,
        first_instance: u32,
        instance_count: u32,
    ) -> MeshDrawCommand {
        MeshDrawCommand::new(
            RenderPhase::Opaque3d,
            MeshPassPipelineKind::Base,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            u64::from(first_instance),
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: first_instance,
                instance_count,
            },
            MeshGeometryHandle::test(7),
            MeshDrawArgs::DirectIndexed {
                first_index,
                index_count,
                first_instance,
                instance_count,
            },
        )
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
