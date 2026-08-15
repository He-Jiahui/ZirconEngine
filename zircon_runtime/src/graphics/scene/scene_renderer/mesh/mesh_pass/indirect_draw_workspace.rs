use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::core::framework::render::RenderCapabilitySummary;
use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;

use super::{
    grow_indirect_buffer_capacity, MeshIndirectCompactionWorkspace, MeshIndirectDrawExecution,
    MeshIndirectDrawPlan, MeshIndirectResourceIdentity, MeshPassIndirectDrawExecutions,
    MeshPassIndirectDrawPlans, INDEXED_INDIRECT_ARGS_STRIDE_BYTES,
};

#[derive(Default)]
pub(crate) struct MeshIndirectDrawWorkspace {
    depth_prepass: MeshIndirectPhaseWorkspace,
    shadow: MeshIndirectPhaseWorkspace,
    opaque: MeshIndirectPhaseWorkspace,
    alpha_mask: MeshIndirectPhaseWorkspace,
    advanced_pbr_opaque: MeshIndirectPhaseWorkspace,
    transparent: MeshIndirectPhaseWorkspace,
    half_resolution_transparent: MeshIndirectPhaseWorkspace,
    velocity: MeshIndirectPhaseWorkspace,
    taa_reactive_mask: MeshIndirectPhaseWorkspace,
}

pub(crate) struct MeshIndirectPhaseWorkspace {
    workspace_id: u64,
    resource_revision: u64,
    args_buffer: Option<Arc<wgpu::Buffer>>,
    args_capacity_bytes: wgpu::BufferAddress,
    args_shadow: Vec<IndexedIndirectArgs>,
    compaction: MeshIndirectCompactionWorkspace,
}

static NEXT_INDIRECT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(1);

impl Default for MeshIndirectPhaseWorkspace {
    fn default() -> Self {
        Self {
            workspace_id: NEXT_INDIRECT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed),
            resource_revision: 0,
            args_buffer: None,
            args_capacity_bytes: 0,
            args_shadow: Vec::new(),
            compaction: MeshIndirectCompactionWorkspace::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshIndirectWorkspaceFrameStats {
    pub(crate) created_buffer_count: u32,
    pub(crate) uploaded_byte_count: u64,
    pub(crate) upload_range_count: u32,
}

impl MeshIndirectDrawWorkspace {
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        capabilities: &RenderCapabilitySummary,
        plans: MeshPassIndirectDrawPlans,
    ) -> (
        MeshPassIndirectDrawExecutions,
        MeshIndirectWorkspaceFrameStats,
    ) {
        let mut stats = MeshIndirectWorkspaceFrameStats::default();
        let depth_prepass = prepare_phase(
            &mut self.depth_prepass,
            device,
            queue,
            "zircon-depth-prepass-indirect-args",
            plans.depth_prepass,
            capabilities,
            &mut stats,
        );
        let shadow = prepare_phase(
            &mut self.shadow,
            device,
            queue,
            "zircon-shadow-indirect-args",
            plans.shadow,
            capabilities,
            &mut stats,
        );
        let opaque = prepare_phase(
            &mut self.opaque,
            device,
            queue,
            "zircon-opaque-indirect-args",
            plans.opaque,
            capabilities,
            &mut stats,
        );
        let alpha_mask = prepare_phase(
            &mut self.alpha_mask,
            device,
            queue,
            "zircon-alpha-mask-indirect-args",
            plans.alpha_mask,
            capabilities,
            &mut stats,
        );
        let advanced_pbr_opaque = prepare_phase(
            &mut self.advanced_pbr_opaque,
            device,
            queue,
            "zircon-advanced-pbr-opaque-indirect-args",
            plans.advanced_pbr_opaque,
            capabilities,
            &mut stats,
        );
        let transparent = prepare_phase(
            &mut self.transparent,
            device,
            queue,
            "zircon-transparent-indirect-args",
            plans.transparent,
            capabilities,
            &mut stats,
        );
        let half_resolution_transparent = prepare_phase(
            &mut self.half_resolution_transparent,
            device,
            queue,
            "zircon-halfres-transparent-indirect-args",
            plans.half_resolution_transparent,
            capabilities,
            &mut stats,
        );
        let velocity = prepare_phase(
            &mut self.velocity,
            device,
            queue,
            "zircon-velocity-indirect-args",
            plans.velocity,
            capabilities,
            &mut stats,
        );
        let taa_reactive_mask = prepare_phase(
            &mut self.taa_reactive_mask,
            device,
            queue,
            "zircon-taa-reactive-mask-indirect-args",
            plans.taa_reactive_mask,
            capabilities,
            &mut stats,
        );

        (
            MeshPassIndirectDrawExecutions::from_phases(
                depth_prepass,
                shadow,
                opaque,
                alpha_mask,
                advanced_pbr_opaque,
                transparent,
                half_resolution_transparent,
                velocity,
                taa_reactive_mask,
            ),
            stats,
        )
    }
}

impl MeshIndirectPhaseWorkspace {
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &'static str,
        plan: MeshIndirectDrawPlan,
        capabilities: &RenderCapabilitySummary,
    ) -> (MeshIndirectDrawExecution, MeshIndirectWorkspaceFrameStats) {
        let args = plan.batcher.args_cpu();
        let required_args_bytes = args.len() as wgpu::BufferAddress
            * std::mem::size_of::<IndexedIndirectArgs>() as wgpu::BufferAddress;
        let required_args_bytes = required_args_bytes.max(INDEXED_INDIRECT_ARGS_STRIDE_BYTES);
        let args_buffer_recreated =
            self.args_buffer.is_none() || self.args_capacity_bytes < required_args_bytes;
        if args_buffer_recreated {
            self.args_capacity_bytes =
                grow_indirect_buffer_capacity(self.args_capacity_bytes, required_args_bytes);
            self.args_buffer = Some(Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: self.args_capacity_bytes,
                usage: indirect_args_usage(),
                mapped_at_creation: false,
            })));
        }

        let mut stats = MeshIndirectWorkspaceFrameStats {
            created_buffer_count: u32::from(args_buffer_recreated),
            uploaded_byte_count: 0,
            upload_range_count: 0,
        };
        let args_upload = super::write_changed_pod_ranges(
            queue,
            self.args_buffer
                .as_ref()
                .expect("indirect args buffer was prepared"),
            &mut self.args_shadow,
            args,
            args_buffer_recreated,
        );
        stats.uploaded_byte_count = args_upload.byte_count;
        stats.upload_range_count = args_upload.range_count;

        let (compaction_resources, compaction_stats) =
            self.compaction
                .prepare(device, queue, label, &plan.compaction_plan);
        stats.created_buffer_count = stats
            .created_buffer_count
            .saturating_add(compaction_stats.created_buffer_count);
        stats.uploaded_byte_count = stats
            .uploaded_byte_count
            .saturating_add(compaction_stats.uploaded_byte_count);
        stats.upload_range_count = stats
            .upload_range_count
            .saturating_add(compaction_stats.upload_range_count);
        if stats.created_buffer_count > 0 {
            self.resource_revision = self.resource_revision.wrapping_add(1).max(1);
        }
        let execution = MeshIndirectDrawExecution::from_prepared_plan(
            MeshIndirectResourceIdentity::new(self.workspace_id, self.resource_revision),
            Arc::clone(
                self.args_buffer
                    .as_ref()
                    .expect("indirect args buffer was prepared"),
            ),
            plan,
            compaction_resources,
            capabilities,
        );
        (execution, stats)
    }
}

fn prepare_phase(
    workspace: &mut MeshIndirectPhaseWorkspace,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    plan: Option<MeshIndirectDrawPlan>,
    capabilities: &RenderCapabilitySummary,
    total_stats: &mut MeshIndirectWorkspaceFrameStats,
) -> Option<MeshIndirectDrawExecution> {
    let plan = plan?;
    let (execution, stats) = workspace.prepare(device, queue, label, plan, capabilities);
    total_stats.created_buffer_count = total_stats
        .created_buffer_count
        .saturating_add(stats.created_buffer_count);
    total_stats.uploaded_byte_count = total_stats
        .uploaded_byte_count
        .saturating_add(stats.uploaded_byte_count);
    total_stats.upload_range_count = total_stats
        .upload_range_count
        .saturating_add(stats.upload_range_count);
    Some(execution)
}

fn indirect_args_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::INDIRECT
        | wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC
}

#[cfg(test)]
mod tests;
