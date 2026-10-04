use std::time::Instant;

use crate::asset::artifact::RenderArtifactIoPriority;
use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
use crate::core::resource::{
    ResourceKind, ResourceManagementGeneration, ResourceReadinessGeneration,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::render_scene::{
    RenderSceneComponentProjectionCommit, RenderSceneComponentProjectionError,
    RenderSceneComponentProjectionTransactionError, RenderSceneComponentProjector,
    RenderSceneRegistry, RenderSceneResourceReferenceDelta,
};
use crate::graphics::types::GraphicsError;
use crate::rhi::SubmissionPollReceipt;

use super::super::{
    RenderAssetDemandGeneration, RenderAssetDeviceEpoch, RenderAssetGpuMaintenanceBudget,
    RenderAssetGpuMaintenanceReport, RenderAssetResidencyAdmissionError,
    RenderAssetResidencyMutation, RenderAssetResidencyRoute, RenderAssetResidencyTicket,
    RenderAssetSemanticExecutorOwner, RenderAssetSemanticExecutorOwnerAdmissionError,
};
use super::render_scene_geometry::PreparedRenderSceneGeometryResolver;
use super::ResourceStreamer;

const MAX_GEOMETRY_REPLAY_PRIMITIVES: usize = 256;

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderSceneFrameAdmissionError {
    #[error(transparent)]
    Geometry(#[from] GraphicsError),
    #[error(transparent)]
    Projection(#[from] RenderSceneComponentProjectionError),
    #[error("render asset residency admission rejected: {0:?}")]
    Residency(RenderAssetResidencyAdmissionError),
    #[error("render frame generation {frame_generation} cannot identify residency demand")]
    InvalidDemandGeneration { frame_generation: u64 },
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderSceneWorldReleaseError {
    #[error(transparent)]
    Asset(#[from] GraphicsError),
    #[error("render asset residency release rejected: {0:?}")]
    Residency(RenderAssetResidencyAdmissionError),
    #[error("render frame generation {frame_generation} cannot identify residency demand")]
    InvalidDemandGeneration { frame_generation: u64 },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderAssetSemanticRoutingBudget {
    max_cancellations: usize,
    max_admissions: usize,
}

impl RenderAssetSemanticRoutingBudget {
    pub(crate) const fn new(max_cancellations: usize, max_admissions: usize) -> Self {
        Self {
            max_cancellations,
            max_admissions,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderAssetSemanticRoutingReport {
    cancellation_attempts: usize,
    cancellations_applied: usize,
    unmatched_cancellations: usize,
    admission_attempts: usize,
    admitted: usize,
    remaining_requests: usize,
    remaining_cancellations: usize,
}

impl RenderAssetSemanticRoutingReport {
    pub(crate) const fn cancellation_attempts(self) -> usize {
        self.cancellation_attempts
    }

    pub(crate) const fn cancellations_applied(self) -> usize {
        self.cancellations_applied
    }

    pub(crate) const fn unmatched_cancellations(self) -> usize {
        self.unmatched_cancellations
    }

    pub(crate) const fn admission_attempts(self) -> usize {
        self.admission_attempts
    }

    pub(crate) const fn admitted(self) -> usize {
        self.admitted
    }

    pub(crate) const fn remaining_requests(self) -> usize {
        self.remaining_requests
    }

    pub(crate) const fn remaining_cancellations(self) -> usize {
        self.remaining_cancellations
    }
}

#[derive(Debug, thiserror::Error)]
#[error("semantic residency owner rejected ticket {ticket:?}: {error}")]
pub(crate) struct RenderAssetSemanticRoutingFailure {
    ticket: RenderAssetResidencyTicket,
    report: RenderAssetSemanticRoutingReport,
    #[source]
    error: RenderAssetSemanticExecutorOwnerAdmissionError,
}

impl RenderAssetSemanticRoutingFailure {
    pub(crate) fn ticket(&self) -> RenderAssetResidencyTicket {
        self.ticket.clone()
    }

    pub(crate) const fn report(&self) -> RenderAssetSemanticRoutingReport {
        self.report
    }

    pub(crate) const fn error(&self) -> &RenderAssetSemanticExecutorOwnerAdmissionError {
        &self.error
    }
}

impl ResourceStreamer {
    /// Applies only an authoritative scene-journal delta. Frame visibility is not a lifetime
    /// source because it cannot distinguish temporarily culled resources from released ones.
    pub(crate) fn apply_render_asset_residency_reference_deltas(
        &mut self,
        deltas: &[RenderSceneResourceReferenceDelta],
        management: &ResourceManagementGeneration,
        readiness: &ResourceReadinessGeneration,
        device: RenderAssetDeviceEpoch,
        demand_generation: RenderAssetDemandGeneration,
    ) -> Result<RenderAssetResidencyMutation, RenderAssetResidencyAdmissionError> {
        let mutation = self.render_asset_residency.apply_scene_reference_deltas(
            deltas,
            management,
            readiness,
            device,
            demand_generation,
        )?;
        self.render_asset_residency_work_queue
            .retain_mutation(&mutation);
        Ok(mutation)
    }

    // 投影成功后才推进本次重放选择；截断批次可提交已完成的投影和驻留变化，但保留 pending 游标供后续帧续接。
    pub(crate) fn admit_render_scene_frame(
        &mut self,
        projector: &mut RenderSceneComponentProjector,
        backend: &RenderBackend,
        frame: &RenderFrameExtract,
        frame_generation: u64,
    ) -> Result<
        Option<RenderSceneComponentProjectionCommit<RenderAssetResidencyMutation>>,
        RenderSceneFrameAdmissionError,
    > {
        let demand_generation = RenderAssetDemandGeneration::new(frame_generation)
            .ok_or(RenderSceneFrameAdmissionError::InvalidDemandGeneration { frame_generation })?;
        let world = frame.world.raw();
        let exact_replay = projector.is_exact_frame_replay(frame)?;
        let artifact_geometry_resources = if exact_replay {
            None
        } else {
            frame
                .geometry
                .scene_changes
                .as_deref()
                .map(RenderSceneComponentProjector::artifact_geometry_resources)
        };
        let artifact_may_reference_supplemental_geometry = !exact_replay
            && frame.geometry.scene_changes.as_deref().is_some_and(
                RenderSceneComponentProjector::artifact_may_reference_supplemental_geometry,
            );
        if !self.geometry_replays.contains_key(&world) {
            let receiver = self.asset_manager()?.resource_manager().subscribe();
            self.geometry_replays.insert(
                world,
                super::geometry_replay::RenderSceneGeometryReplayState::new(Some(receiver)),
            );
        } else if self
            .geometry_replays
            .get(&world)
            .is_some_and(super::geometry_replay::RenderSceneGeometryReplayState::receiver_missing)
        {
            let receiver = self.asset_manager()?.resource_manager().subscribe();
            self.geometry_replays
                .get_mut(&world)
                .expect("world geometry replay cursor was inserted")
                .install_receiver(receiver);
        }
        let replay = self
            .geometry_replays
            .get_mut(&world)
            .expect("world geometry replay cursor was inserted")
            .drain(MAX_GEOMETRY_REPLAY_PRIMITIVES, |resource| {
                projector.has_geometry_dependency(resource)
                    || artifact_geometry_resources
                        .as_ref()
                        .is_some_and(|resources| resources.contains(&resource))
                    || (artifact_may_reference_supplemental_geometry
                        && matches!(resource.kind(), ResourceKind::Mesh | ResourceKind::Model))
            });
        let has_resource_replay = replay.resync || !replay.resources.is_empty();
        if exact_replay && !has_resource_replay {
            return Ok(Some(RenderSceneComponentProjectionCommit::Replayed));
        }
        if !has_resource_replay && frame.geometry.scene_changes.is_none() {
            return Ok(None);
        }
        if !exact_replay {
            if let Some(artifact) = frame.geometry.scene_changes.as_deref() {
                self.ensure_render_scene_projection_geometry(&backend.device, artifact)?;
            }
        }
        let resources = if replay.resync {
            Vec::new()
        } else {
            replay.resources
        };
        let replay_limit = MAX_GEOMETRY_REPLAY_PRIMITIVES;
        let selection = projector.prepare_geometry_replay_with_revisions(
            &resources,
            replay.resync,
            replay_limit,
            (!exact_replay)
                .then_some(frame.geometry.scene_changes.as_deref())
                .flatten(),
        );
        if has_resource_replay {
            self.ensure_render_scene_projection_geometry_resources(
                &backend.device,
                selection.required_resources().iter().copied(),
            )?;
        }
        let asset_manager = self.asset_manager()?;
        let projection = asset_manager.resource_manager().projection_snapshot();
        let profile = backend.device_profile();
        let device = RenderAssetDeviceEpoch::new(profile.device_id(), profile.generation());
        let Self {
            models,
            meshes,
            render_asset_residency,
            render_asset_residency_work_queue,
            ..
        } = self;
        let mut resolver = PreparedRenderSceneGeometryResolver::new(models, meshes);
        let result = projector
            .project_frame_with_resource_geometry_staging(
                frame,
                &selection,
                &mut resolver,
                |deltas| {
                    let mutation = render_asset_residency.apply_scene_reference_deltas(
                        deltas,
                        projection.management(),
                        projection.readiness(),
                        device,
                        demand_generation,
                    )?;
                    render_asset_residency_work_queue.retain_mutation(&mutation);
                    Ok(mutation)
                },
            )
            .map_err(|error| match error {
                RenderSceneComponentProjectionTransactionError::Projection(error) => {
                    RenderSceneFrameAdmissionError::Projection(error)
                }
                RenderSceneComponentProjectionTransactionError::Staging(error) => {
                    RenderSceneFrameAdmissionError::Residency(error)
                }
            })?;
        if has_resource_replay {
            projector.finish_geometry_replay_selection(&selection);
            self.geometry_replays
                .get_mut(&world)
                .expect("world geometry replay cursor must survive admission")
                .commit(projector.geometry_resync_complete(), !selection.truncated());
        }
        if has_resource_replay || !exact_replay {
            self.geometry_replays
                .get_mut(&world)
                .expect("world geometry replay cursor must survive admission")
                .retain_dependencies(|resource| projector.has_geometry_dependency(resource));
        }
        Ok(result)
    }

    pub(crate) fn pending_render_asset_residency_request_count(&self) -> usize {
        self.render_asset_residency_work_queue
            .pending_request_count()
    }

    pub(crate) fn route_pending_render_asset_semantic_requests(
        &mut self,
        owner: &mut RenderAssetSemanticExecutorOwner,
        budget: RenderAssetSemanticRoutingBudget,
        priority: RenderArtifactIoPriority,
        deadline: Option<Instant>,
    ) -> Result<RenderAssetSemanticRoutingReport, RenderAssetSemanticRoutingFailure> {
        let Self {
            render_asset_residency,
            render_asset_residency_work_queue,
            ..
        } = self;
        let mut report = RenderAssetSemanticRoutingReport::default();

        for _ in 0..budget.max_cancellations {
            let Some(ticket) = render_asset_residency_work_queue.pop_next_semantic_cancellation()
            else {
                break;
            };
            report.cancellation_attempts = report.cancellation_attempts.saturating_add(1);
            if owner.cancel(&ticket) {
                report.cancellations_applied = report.cancellations_applied.saturating_add(1);
            } else {
                report.unmatched_cancellations = report.unmatched_cancellations.saturating_add(1);
                render_asset_residency_work_queue.restore_semantic_cancellation_front(ticket);
                break;
            }
        }

        for _ in 0..budget.max_admissions {
            let admission = render_asset_residency_work_queue.try_admit_next_semantic(|ticket| {
                owner.admit(ticket, priority, deadline, render_asset_residency)
            });
            match admission {
                Ok(Some(_)) => {
                    report.admission_attempts = report.admission_attempts.saturating_add(1);
                    report.admitted = report.admitted.saturating_add(1);
                }
                Ok(None) => break,
                Err(failure) => {
                    report.admission_attempts = report.admission_attempts.saturating_add(1);
                    report.remaining_requests = render_asset_residency_work_queue
                        .pending_request_count_for_route(RenderAssetResidencyRoute::SemanticBlocks);
                    report.remaining_cancellations =
                        render_asset_residency_work_queue.pending_semantic_cancellation_count();
                    let (ticket, error) = failure.into_parts();
                    return Err(RenderAssetSemanticRoutingFailure {
                        ticket,
                        report,
                        error,
                    });
                }
            }
        }

        report.remaining_requests = render_asset_residency_work_queue
            .pending_request_count_for_route(RenderAssetResidencyRoute::SemanticBlocks);
        report.remaining_cancellations =
            render_asset_residency_work_queue.pending_semantic_cancellation_count();
        Ok(report)
    }

    pub(crate) fn pending_render_asset_semantic_cancellation_count(&self) -> usize {
        self.render_asset_residency_work_queue
            .pending_semantic_cancellation_count()
    }

    /// Retires one explicitly unloaded world after its complete resource release is accepted.
    pub(crate) fn release_render_scene_world(
        &mut self,
        registry: &mut RenderSceneRegistry,
        backend: &RenderBackend,
        world: RenderWorldSnapshotHandle,
        frame_generation: u64,
    ) -> Result<bool, RenderSceneWorldReleaseError> {
        let demand_generation = RenderAssetDemandGeneration::new(frame_generation)
            .ok_or(RenderSceneWorldReleaseError::InvalidDemandGeneration { frame_generation })?;
        let asset_manager = self.asset_manager()?;
        let projection = asset_manager.resource_manager().projection_snapshot();
        let profile = backend.device_profile();
        let device = RenderAssetDeviceEpoch::new(profile.device_id(), profile.generation());
        let Self {
            render_asset_residency,
            render_asset_residency_work_queue,
            ..
        } = self;
        let released = registry
            .release_world_with_staging(world, |deltas| {
                let mutation = render_asset_residency
                    .apply_scene_reference_deltas(
                        deltas,
                        projection.management(),
                        projection.readiness(),
                        device,
                        demand_generation,
                    )
                    .map_err(RenderSceneWorldReleaseError::Residency)?;
                render_asset_residency_work_queue.retain_mutation(&mutation);
                Ok::<_, RenderSceneWorldReleaseError>(mutation)
            })?
            .is_some();
        if released {
            self.geometry_replays.remove(&world.raw());
        }
        Ok(released)
    }

    pub(crate) fn pending_render_asset_residency_retirement_count(&self) -> usize {
        self.render_asset_residency
            .retiring_gpu_upload_count()
            .saturating_add(self.render_asset_residency.ready_gpu_retirement_count())
    }

    /// Routes the backend owner's already-completed poll through the residency timeline.
    /// This function never polls the device and therefore preserves the single-poll frame owner.
    pub(crate) fn maintain_render_asset_gpu_residency_after_rhi_poll(
        &mut self,
        backend: &RenderBackend,
        poll_receipt: SubmissionPollReceipt,
    ) {
        let terminal_capacity = backend
            .device_profile()
            .submission_limits()
            .max_terminal_statuses();
        let report = self.render_asset_residency.maintain_gpu_after_rhi_poll(
            backend.render_device.as_ref(),
            poll_receipt,
            RenderAssetGpuMaintenanceBudget::new(terminal_capacity, terminal_capacity),
        );
        crate::profile_counter!(
            "render",
            "render_asset_residency_submission_status_checks",
            report.submission_status_checks() as u64
        );
        crate::profile_counter!(
            "render",
            "render_asset_residency_published_artifacts",
            report.published_artifacts() as u64
        );
        crate::profile_counter!(
            "render",
            "render_asset_residency_failed_uploads",
            report.failed_uploads() as u64
        );
        crate::profile_counter!(
            "render",
            "render_asset_residency_retired_bytes",
            report.retired_bytes()
        );
        crate::profile_counter!(
            "render",
            "render_asset_residency_maintenance_failures",
            report.failures().len() as u64
        );
        self.last_render_asset_gpu_maintenance = report;
    }

    pub(crate) fn last_render_asset_gpu_maintenance_report(
        &self,
    ) -> &RenderAssetGpuMaintenanceReport {
        &self.last_render_asset_gpu_maintenance
    }
}

#[cfg(test)]
#[path = "tests/resource_streamer_residency.rs"]
mod tests;
