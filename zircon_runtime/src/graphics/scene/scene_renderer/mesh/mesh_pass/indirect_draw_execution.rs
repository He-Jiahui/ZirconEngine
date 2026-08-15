use std::cell::Cell;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::RenderCapabilitySummary;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;
use zr_rhi_wgpu::{GpuReadbackQueue, ReadbackError};

use super::{
    IndirectCompactionPlan, IndirectDrawBatch, MeshDrawCommand, MeshIndirectCompactionResources,
    MeshIndirectDrawPlan,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MeshIndirectResourceIdentity {
    workspace_id: u64,
    resource_revision: u64,
}

impl MeshIndirectResourceIdentity {
    pub(crate) const fn new(workspace_id: u64, resource_revision: u64) -> Self {
        Self {
            workspace_id,
            resource_revision,
        }
    }

    pub(crate) const fn workspace_id(self) -> u64 {
        self.workspace_id
    }

    pub(crate) const fn resource_revision(self) -> u64 {
        self.resource_revision
    }
}

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
    resource_identity: MeshIndirectResourceIdentity,
    args_buffer: Arc<wgpu::Buffer>,
    batches: Vec<IndirectDrawBatch>,
    compaction_plan: IndirectCompactionPlan,
    compaction_resources: MeshIndirectCompactionResources,
    multi_draw_indirect_supported: bool,
    indirect_count_supported: bool,
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
    args: SharedReadbackBytes,
    args_count: u32,
    draw_counts: Option<SharedReadbackBytes>,
    draw_count_count: u32,
}

#[derive(Clone, Default)]
struct SharedReadbackBytes {
    result: Arc<Mutex<Option<Result<Vec<u8>, String>>>>,
}

impl MeshIndirectDrawExecution {
    pub(crate) fn from_prepared_plan(
        resource_identity: MeshIndirectResourceIdentity,
        args_buffer: Arc<wgpu::Buffer>,
        plan: MeshIndirectDrawPlan,
        compaction_resources: MeshIndirectCompactionResources,
        capabilities: &RenderCapabilitySummary,
    ) -> Self {
        let compaction_plan = plan.compaction_plan;
        let (args_cpu, batches) = plan.batcher.into_execution_parts();
        let args_count = args_cpu.len() as u32;
        let total_instances = batches.iter().map(|batch| batch.total_instances).sum();

        Self {
            resource_identity,
            args_buffer,
            batches,
            compaction_plan,
            compaction_resources,
            multi_draw_indirect_supported: capabilities.supports_multi_draw_indirect,
            indirect_count_supported: capabilities.gpu_driven_indirect_count_supported(),
            visible_remap_scene_bind_group: None,
            compaction_ready_for_replay: Cell::new(false),
            args_count,
            total_instances,
        }
    }

    pub(crate) fn args_buffer(&self) -> &wgpu::Buffer {
        &self.args_buffer
    }

    pub(crate) const fn resource_identity(&self) -> MeshIndirectResourceIdentity {
        self.resource_identity
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

    pub(crate) const fn indirect_count_supported(&self) -> bool {
        self.indirect_count_supported
    }

    pub(crate) const fn multi_draw_indirect_supported(&self) -> bool {
        self.multi_draw_indirect_supported
    }

    pub(crate) const fn args_count(&self) -> u32 {
        self.args_count
    }

    pub(crate) const fn total_instances(&self) -> u32 {
        self.total_instances
    }

    pub(crate) fn request_args_readback(
        &self,
        queue: &mut GpuReadbackQueue,
        label: &'static str,
    ) -> Result<MeshIndirectArgsReadback, ReadbackError> {
        let byte_size = self.args_readback_byte_size();
        let args = SharedReadbackBytes::default();
        args.request(queue, label, self.replay_args_buffer(), byte_size)?;

        let draw_count_count = self.compaction_resources.draw_count_capacity();
        let draw_count_byte_size = self.compaction_resources.draw_count_buffer_byte_size();
        let draw_counts = (self.compaction_ready_for_replay()
            && draw_count_count > 0
            && draw_count_byte_size > 0)
            .then(|| -> Result<_, ReadbackError> {
                let readback = SharedReadbackBytes::default();
                readback.request(
                    queue,
                    label,
                    self.compaction_resources.draw_count_buffer(),
                    draw_count_byte_size,
                )?;
                Ok(readback)
            })
            .transpose()?;
        let copied_draw_count_count = if draw_counts.is_some() {
            draw_count_count.min((draw_count_byte_size / DRAW_COUNT_STRIDE_BYTES) as u32)
        } else {
            0
        };

        Ok(MeshIndirectArgsReadback {
            args,
            args_count: self.args_count,
            draw_counts,
            draw_count_count: copied_draw_count_count,
        })
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
    pub(crate) fn is_ready(&self) -> bool {
        self.args.is_ready()
            && self
                .draw_counts
                .as_ref()
                .map_or(true, SharedReadbackBytes::is_ready)
    }

    pub(crate) fn collect(self) -> Option<MeshIndirectArgsSnapshot> {
        let args = decode_indexed_indirect_args(&self.args.take()?, self.args_count)?;
        let draw_counts = if let Some(draw_counts) = self.draw_counts {
            decode_u32s(&draw_counts.take()?, self.draw_count_count)?
        } else {
            Vec::new()
        };

        Some(MeshIndirectArgsSnapshot::from_args_and_draw_counts(
            args,
            draw_counts,
        ))
    }
}

impl SharedReadbackBytes {
    fn request(
        &self,
        queue: &mut GpuReadbackQueue,
        name: impl Into<String>,
        source: &wgpu::Buffer,
        byte_size: u64,
    ) -> Result<(), ReadbackError> {
        let result = Arc::clone(&self.result);
        queue.request_readback_external(
            name,
            source,
            0..byte_size,
            Box::new(move |readback| {
                *result
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(
                    readback
                        .map(<[u8]>::to_vec)
                        .map_err(|error| error.to_string()),
                );
            }),
        )?;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
    }

    fn take(self) -> Option<Vec<u8>> {
        let mut result = self
            .result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        result.take()?.ok()
    }
}

fn decode_indexed_indirect_args(bytes: &[u8], count: u32) -> Option<Vec<IndexedIndirectArgs>> {
    let byte_len = count as usize * INDEXED_INDIRECT_ARGS_STRIDE_BYTES as usize;
    let bytes = bytes.get(..byte_len)?;
    Some(
        bytes
            .chunks_exact(INDEXED_INDIRECT_ARGS_STRIDE_BYTES as usize)
            .map(|args| IndexedIndirectArgs {
                index_count: decode_u32(args, 0),
                instance_count: decode_u32(args, 4),
                first_index: decode_u32(args, 8),
                base_vertex: decode_u32(args, 12) as i32,
                first_instance: decode_u32(args, 16),
            })
            .collect(),
    )
}

fn decode_u32s(bytes: &[u8], count: u32) -> Option<Vec<u32>> {
    let byte_len = count as usize * std::mem::size_of::<u32>();
    Some(
        bytes
            .get(..byte_len)?
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
            .collect(),
    )
}

fn decode_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

#[derive(Default)]
pub(crate) struct MeshPassIndirectDrawExecutions {
    depth_prepass: Option<MeshIndirectDrawExecution>,
    shadow: Option<MeshIndirectDrawExecution>,
    opaque: Option<MeshIndirectDrawExecution>,
    alpha_mask: Option<MeshIndirectDrawExecution>,
    advanced_pbr_opaque: Option<MeshIndirectDrawExecution>,
    transparent: Option<MeshIndirectDrawExecution>,
    half_resolution_transparent: Option<MeshIndirectDrawExecution>,
    velocity: Option<MeshIndirectDrawExecution>,
    taa_reactive_mask: Option<MeshIndirectDrawExecution>,
}

impl MeshPassIndirectDrawExecutions {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_phases(
        depth_prepass: Option<MeshIndirectDrawExecution>,
        shadow: Option<MeshIndirectDrawExecution>,
        opaque: Option<MeshIndirectDrawExecution>,
        alpha_mask: Option<MeshIndirectDrawExecution>,
        advanced_pbr_opaque: Option<MeshIndirectDrawExecution>,
        transparent: Option<MeshIndirectDrawExecution>,
        half_resolution_transparent: Option<MeshIndirectDrawExecution>,
        velocity: Option<MeshIndirectDrawExecution>,
        taa_reactive_mask: Option<MeshIndirectDrawExecution>,
    ) -> Self {
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

    pub(crate) fn half_resolution_transparent(&self) -> Option<&MeshIndirectDrawExecution> {
        self.half_resolution_transparent.as_ref()
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

    pub(crate) fn request_hzb_occlusion_args_readbacks(
        &self,
        queue: &mut GpuReadbackQueue,
        label: &'static str,
    ) -> Result<Vec<MeshIndirectArgsReadback>, ReadbackError> {
        self.hzb_occlusion_executions()
            .into_iter()
            .flatten()
            .map(|execution| execution.request_args_readback(queue, label))
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

    fn executions_mut(&mut self) -> [Option<&mut MeshIndirectDrawExecution>; 9] {
        [
            self.depth_prepass.as_mut(),
            self.shadow.as_mut(),
            self.opaque.as_mut(),
            self.alpha_mask.as_mut(),
            self.advanced_pbr_opaque.as_mut(),
            self.transparent.as_mut(),
            self.half_resolution_transparent.as_mut(),
            self.velocity.as_mut(),
            self.taa_reactive_mask.as_mut(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{MeshIndirectArgsSnapshot, INDEXED_INDIRECT_ARGS_STRIDE_BYTES};
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
        assert!(source.contains("from_prepared_plan"));
        assert!(source.contains("args_buffer: Arc<wgpu::Buffer>"));
        assert!(!source.contains("create_buffer_init"));
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
    fn mesh_indirect_draw_execution_routes_readback_through_the_shared_queue() {
        let source = include_str!("indirect_draw_execution.rs");

        assert!(source.contains("request_readback_external"));
        assert!(source.contains("self.replay_args_buffer()"));
        assert!(source.contains("self.compaction_resources.draw_count_buffer()"));
        assert!(source.contains("SharedReadbackBytes"));
        assert!(source.contains("decode_indexed_indirect_args"));
        assert!(!source.contains("map_async"));
    }

    #[test]
    fn mesh_indirect_draw_execution_builds_compaction_plan_from_uploaded_args() {
        let Some(backend) = crate::graphics::backend::RenderBackend::new_offscreen().ok() else {
            return;
        };
        let commands = vec![command(10, 1, 2, 3), command(20, 4, 8, 2)];

        let (plan, _) =
            super::super::MeshIndirectDrawPlan::build(&commands, &gpu_driven_capabilities());
        let mut workspace = super::super::MeshIndirectPhaseWorkspace::default();
        let (execution, _) = workspace.prepare(
            &backend.device,
            &backend.queue,
            "zircon-test-indirect-compaction-execution",
            plan.expect("indirect plan"),
            &gpu_driven_capabilities(),
        );

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
