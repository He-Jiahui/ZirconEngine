use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::framework::render::RenderCapabilitySummary;
use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::{
    INDEXED_INDIRECT_ARGS_STRIDE_BYTES, MeshIndirectCompactionWorkspace, MeshIndirectDrawExecution,
    MeshIndirectDrawPlan, MeshIndirectResourceIdentity, MeshPassIndirectDrawExecutions,
    MeshPassIndirectDrawPlans, PodRangeUploadCommit, PodRangeUploadShadow,
    grow_indirect_buffer_capacity,
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
    args_buffer_revision: u64,
    args_shadow: PodRangeUploadShadow<IndexedIndirectArgs>,
    compaction: MeshIndirectCompactionWorkspace,
}

#[derive(Clone, Copy)]
enum MeshIndirectPhase {
    DepthPrepass,
    Shadow,
    Opaque,
    AlphaMask,
    AdvancedPbrOpaque,
    Transparent,
    HalfResolutionTransparent,
    Velocity,
    TaaReactiveMask,
}

struct MeshIndirectPhasePreparedCommit {
    phase: MeshIndirectPhase,
    workspace_id: u64,
    resource_revision: u64,
    args: Option<PodRangeUploadCommit>,
    compaction_metadata: Option<PodRangeUploadCommit>,
}

#[derive(Default)]
pub(crate) struct MeshIndirectWorkspacePreparedUpload {
    uploads: WgpuBufferUploadBatch,
    commits: Vec<MeshIndirectPhasePreparedCommit>,
    appended_to_frame: bool,
}

impl MeshIndirectWorkspacePreparedUpload {
    pub(crate) fn append_to(&mut self, frame_uploads: &mut WgpuBufferUploadBatch) {
        assert!(
            !self.appended_to_frame,
            "mesh indirect uploads must be appended to one frame transaction exactly once"
        );
        frame_uploads.append(&mut self.uploads);
        self.appended_to_frame = true;
    }

    pub(crate) fn commit_count(&self) -> usize {
        self.commits.len()
    }

    pub(crate) fn commit(self, workspace: &mut MeshIndirectDrawWorkspace) -> u32 {
        assert!(
            self.appended_to_frame,
            "mesh indirect shadows require an accepted frame upload batch"
        );
        let mut committed_count = 0_u32;
        for commit in self.commits {
            let phase_workspace = workspace.phase_workspace_mut(commit.phase);
            assert!(
                phase_workspace.commit_prepared_upload(commit),
                "mesh indirect upload commit token must match its prepared workspace revision"
            );
            committed_count = committed_count.saturating_add(1);
        }
        committed_count
    }
}

static NEXT_INDIRECT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(1);

impl Default for MeshIndirectPhaseWorkspace {
    fn default() -> Self {
        Self {
            workspace_id: NEXT_INDIRECT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed),
            resource_revision: 0,
            args_buffer: None,
            args_capacity_bytes: 0,
            args_buffer_revision: 0,
            args_shadow: PodRangeUploadShadow::default(),
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
        capabilities: &RenderCapabilitySummary,
        plans: MeshPassIndirectDrawPlans,
    ) -> (
        MeshPassIndirectDrawExecutions,
        MeshIndirectWorkspaceFrameStats,
        MeshIndirectWorkspacePreparedUpload,
    ) {
        let mut stats = MeshIndirectWorkspaceFrameStats::default();
        let mut prepared_upload = MeshIndirectWorkspacePreparedUpload::default();
        let depth_prepass = prepare_phase(
            MeshIndirectPhase::DepthPrepass,
            &mut self.depth_prepass,
            device,
            "zircon-depth-prepass-indirect-args",
            plans.depth_prepass,
            capabilities,
            &mut stats,
            &mut prepared_upload,
        );
        let shadow = prepare_phase(
            MeshIndirectPhase::Shadow,
            &mut self.shadow,
            device,
            "zircon-shadow-indirect-args",
            plans.shadow,
            capabilities,
            &mut stats,
            &mut prepared_upload,
        );
        let opaque = prepare_phase(
            MeshIndirectPhase::Opaque,
            &mut self.opaque,
            device,
            "zircon-opaque-indirect-args",
            plans.opaque,
            capabilities,
            &mut stats,
            &mut prepared_upload,
        );
        let alpha_mask = prepare_phase(
            MeshIndirectPhase::AlphaMask,
            &mut self.alpha_mask,
            device,
            "zircon-alpha-mask-indirect-args",
            plans.alpha_mask,
            capabilities,
            &mut stats,
            &mut prepared_upload,
        );
        let advanced_pbr_opaque = prepare_phase(
            MeshIndirectPhase::AdvancedPbrOpaque,
            &mut self.advanced_pbr_opaque,
            device,
            "zircon-advanced-pbr-opaque-indirect-args",
            plans.advanced_pbr_opaque,
            capabilities,
            &mut stats,
            &mut prepared_upload,
        );
        let transparent = prepare_phase(
            MeshIndirectPhase::Transparent,
            &mut self.transparent,
            device,
            "zircon-transparent-indirect-args",
            plans.transparent,
            capabilities,
            &mut stats,
            &mut prepared_upload,
        );
        let half_resolution_transparent = prepare_phase(
            MeshIndirectPhase::HalfResolutionTransparent,
            &mut self.half_resolution_transparent,
            device,
            "zircon-halfres-transparent-indirect-args",
            plans.half_resolution_transparent,
            capabilities,
            &mut stats,
            &mut prepared_upload,
        );
        let velocity = prepare_phase(
            MeshIndirectPhase::Velocity,
            &mut self.velocity,
            device,
            "zircon-velocity-indirect-args",
            plans.velocity,
            capabilities,
            &mut stats,
            &mut prepared_upload,
        );
        let taa_reactive_mask = prepare_phase(
            MeshIndirectPhase::TaaReactiveMask,
            &mut self.taa_reactive_mask,
            device,
            "zircon-taa-reactive-mask-indirect-args",
            plans.taa_reactive_mask,
            capabilities,
            &mut stats,
            &mut prepared_upload,
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
            prepared_upload,
        )
    }

    fn phase_workspace_mut(&mut self, phase: MeshIndirectPhase) -> &mut MeshIndirectPhaseWorkspace {
        match phase {
            MeshIndirectPhase::DepthPrepass => &mut self.depth_prepass,
            MeshIndirectPhase::Shadow => &mut self.shadow,
            MeshIndirectPhase::Opaque => &mut self.opaque,
            MeshIndirectPhase::AlphaMask => &mut self.alpha_mask,
            MeshIndirectPhase::AdvancedPbrOpaque => &mut self.advanced_pbr_opaque,
            MeshIndirectPhase::Transparent => &mut self.transparent,
            MeshIndirectPhase::HalfResolutionTransparent => &mut self.half_resolution_transparent,
            MeshIndirectPhase::Velocity => &mut self.velocity,
            MeshIndirectPhase::TaaReactiveMask => &mut self.taa_reactive_mask,
        }
    }
}

impl MeshIndirectPhaseWorkspace {
    fn prepare(
        &mut self,
        phase: MeshIndirectPhase,
        device: &wgpu::Device,
        label: &'static str,
        plan: MeshIndirectDrawPlan,
        capabilities: &RenderCapabilitySummary,
        uploads: &mut WgpuBufferUploadBatch,
    ) -> (
        MeshIndirectDrawExecution,
        MeshIndirectWorkspaceFrameStats,
        Option<MeshIndirectPhasePreparedCommit>,
    ) {
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
            self.args_buffer_revision = self.args_buffer_revision.wrapping_add(1).max(1);
        }

        let mut stats = MeshIndirectWorkspaceFrameStats {
            created_buffer_count: u32::from(args_buffer_recreated),
            uploaded_byte_count: 0,
            upload_range_count: 0,
        };
        let (args_upload, args_commit) = self.args_shadow.prepare(
            self.args_buffer
                .as_ref()
                .expect("indirect args buffer was prepared"),
            self.args_buffer_revision,
            args,
            uploads,
        );
        stats.uploaded_byte_count = args_upload.byte_count;
        stats.upload_range_count = args_upload.range_count;

        let (compaction_resources, compaction_stats, compaction_metadata_commit) = self
            .compaction
            .prepare(device, label, &plan.compaction_plan, uploads);
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
        let prepared_commit = (args_commit.is_some() || compaction_metadata_commit.is_some())
            .then_some(MeshIndirectPhasePreparedCommit {
                phase,
                workspace_id: self.workspace_id,
                resource_revision: self.resource_revision,
                args: args_commit,
                compaction_metadata: compaction_metadata_commit,
            });
        (execution, stats, prepared_commit)
    }

    fn commit_prepared_upload(&mut self, commit: MeshIndirectPhasePreparedCommit) -> bool {
        if self.workspace_id != commit.workspace_id
            || self.resource_revision != commit.resource_revision
            || commit
                .args
                .is_some_and(|token| !self.args_shadow.accepts(token))
            || commit
                .compaction_metadata
                .is_some_and(|token| !self.compaction.accepts_metadata_upload(token))
        {
            return false;
        }
        if let Some(token) = commit.args {
            let accepted = self.args_shadow.commit(token);
            debug_assert!(accepted);
        }
        if let Some(token) = commit.compaction_metadata {
            let accepted = self.compaction.commit_metadata_upload(token);
            debug_assert!(accepted);
        }
        true
    }
}

fn prepare_phase(
    phase: MeshIndirectPhase,
    workspace: &mut MeshIndirectPhaseWorkspace,
    device: &wgpu::Device,
    label: &'static str,
    plan: Option<MeshIndirectDrawPlan>,
    capabilities: &RenderCapabilitySummary,
    total_stats: &mut MeshIndirectWorkspaceFrameStats,
    prepared_upload: &mut MeshIndirectWorkspacePreparedUpload,
) -> Option<MeshIndirectDrawExecution> {
    let plan = plan?;
    let (execution, stats, commit) = workspace.prepare(
        phase,
        device,
        label,
        plan,
        capabilities,
        &mut prepared_upload.uploads,
    );
    total_stats.created_buffer_count = total_stats
        .created_buffer_count
        .saturating_add(stats.created_buffer_count);
    total_stats.uploaded_byte_count = total_stats
        .uploaded_byte_count
        .saturating_add(stats.uploaded_byte_count);
    total_stats.upload_range_count = total_stats
        .upload_range_count
        .saturating_add(stats.upload_range_count);
    if let Some(commit) = commit {
        prepared_upload.commits.push(commit);
    }
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
