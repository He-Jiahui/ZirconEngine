use std::sync::Arc;

use crate::asset::{ProjectAssetManager, TextureAsset};
use crate::core::framework::render::RenderFrameSubmissionTransaction;
use crate::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManagementGeneration,
    ResourceManagementGenerationIdentity, ResourceManagementQuery, ResourceManagementRow,
    ResourceReadinessGenerationIdentity, ResourceReadinessState,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::resources::{ResourceStreamer, TextureSnapshotFramePrepareError};

use super::{UiTextureDependencies, is_ui_texture_descriptor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiTexturePrepareOutcome {
    UnresolvedIdentity,
    NotReady,
    LoadFailed,
    InvalidResourceKind,
    InvalidDescriptor,
    GenerationChanged,
    UploadFailed,
    Ready,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiTexturePrepareRow {
    pub(super) requested: ResourceId,
    pub(super) resolved: Option<ResourceId>,
    pub(super) outcome: UiTexturePrepareOutcome,
    pub(super) prepared_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct UiTexturePrepareSummary {
    requested_count: usize,
    ready_count: usize,
    unresolved_count: usize,
    not_ready_count: usize,
    load_failed_count: usize,
    invalid_resource_kind_count: usize,
    invalid_descriptor_count: usize,
    generation_changed_count: usize,
    upload_failed_count: usize,
}

#[derive(Clone, Debug)]
pub(in crate::graphics::scene) struct UiTexturePrepareReceipt {
    frame_prepare_epoch: u64,
    management_generation: ResourceManagementGenerationIdentity,
    readiness_generation: ResourceReadinessGenerationIdentity,
    rows: Arc<[UiTexturePrepareRow]>,
    summary: UiTexturePrepareSummary,
    work: UiTexturePrepareWork,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct UiTexturePrepareWork {
    resolution_scan_row_visit_count: usize,
    snapshot_load_count: usize,
    prepared_reuse_count: usize,
    upload_attempt_count: usize,
}

impl UiTexturePrepareReceipt {
    pub(super) fn new(
        frame_prepare_epoch: u64,
        management_generation: ResourceManagementGenerationIdentity,
        readiness_generation: ResourceReadinessGenerationIdentity,
        mut rows: Vec<UiTexturePrepareRow>,
    ) -> Self {
        rows.sort_unstable_by_key(|row| row.requested);
        debug_assert!(
            rows.windows(2)
                .all(|pair| pair[0].requested != pair[1].requested)
        );
        let summary = UiTexturePrepareSummary::from_rows(&rows);
        Self {
            frame_prepare_epoch,
            management_generation,
            readiness_generation,
            rows: rows.into(),
            summary,
            work: UiTexturePrepareWork::default(),
        }
    }

    fn with_work(mut self, work: UiTexturePrepareWork) -> Self {
        self.work = work;
        self
    }

    pub(in crate::graphics::scene) const fn frame_prepare_epoch(&self) -> u64 {
        self.frame_prepare_epoch
    }

    pub(in crate::graphics::scene) fn management_generation(
        &self,
    ) -> &ResourceManagementGenerationIdentity {
        &self.management_generation
    }

    pub(in crate::graphics::scene) fn readiness_generation(
        &self,
    ) -> &ResourceReadinessGenerationIdentity {
        &self.readiness_generation
    }

    pub(in crate::graphics::scene) fn ready_texture_id(
        &self,
        requested: ResourceId,
    ) -> Option<ResourceId> {
        self.ready_texture_binding(requested).map(|(id, _)| id)
    }

    pub(in crate::graphics::scene) fn ready_texture_binding(
        &self,
        requested: ResourceId,
    ) -> Option<(ResourceId, u64)> {
        let row = self
            .rows
            .binary_search_by_key(&requested, |row| row.requested)
            .ok()
            .and_then(|index| self.rows.get(index))?;
        if row.outcome != UiTexturePrepareOutcome::Ready {
            return None;
        }
        Some((row.resolved?, row.prepared_revision?))
    }

    fn record_profile_counters(&self) {
        let summary = self.summary;
        let work = self.work;
        crate::core::diagnostics::profiling::record_counter_batch(
            "runtime",
            &[
                (
                    "ui.ui_texture_prepare.requested_count",
                    summary.requested_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.ready_count",
                    summary.ready_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.unresolved_count",
                    summary.unresolved_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.not_ready_count",
                    summary.not_ready_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.load_failed_count",
                    summary.load_failed_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.invalid_resource_kind_count",
                    summary.invalid_resource_kind_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.invalid_descriptor_count",
                    summary.invalid_descriptor_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.generation_changed_count",
                    summary.generation_changed_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.upload_failed_count",
                    summary.upload_failed_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.resolution_scan_row_visit_count",
                    work.resolution_scan_row_visit_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.snapshot_load_count",
                    work.snapshot_load_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.prepared_reuse_count",
                    work.prepared_reuse_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.upload_attempt_count",
                    work.upload_attempt_count as f64,
                ),
            ],
        );
    }
}

impl UiTexturePrepareSummary {
    fn from_rows(rows: &[UiTexturePrepareRow]) -> Self {
        let mut summary = Self {
            requested_count: rows.len(),
            ..Self::default()
        };
        for row in rows {
            match row.outcome {
                UiTexturePrepareOutcome::UnresolvedIdentity => summary.unresolved_count += 1,
                UiTexturePrepareOutcome::NotReady => summary.not_ready_count += 1,
                UiTexturePrepareOutcome::LoadFailed => summary.load_failed_count += 1,
                UiTexturePrepareOutcome::InvalidResourceKind => {
                    summary.invalid_resource_kind_count += 1;
                }
                UiTexturePrepareOutcome::InvalidDescriptor => {
                    summary.invalid_descriptor_count += 1;
                }
                UiTexturePrepareOutcome::GenerationChanged => {
                    summary.generation_changed_count += 1;
                }
                UiTexturePrepareOutcome::UploadFailed => summary.upload_failed_count += 1,
                UiTexturePrepareOutcome::Ready => summary.ready_count += 1,
            }
        }
        summary
    }
}

pub(super) fn resolve_ui_texture_candidate(
    generation: &Arc<ResourceManagementGeneration>,
    requested: ResourceId,
) -> Result<Arc<ResourceManagementRow>, UiTexturePrepareOutcome> {
    resolve_ui_texture_candidate_with_work(generation, requested).0
}

fn resolve_ui_texture_candidate_with_work(
    generation: &Arc<ResourceManagementGeneration>,
    requested: ResourceId,
) -> (
    Result<Arc<ResourceManagementRow>, UiTexturePrepareOutcome>,
    Option<ResourceId>,
    usize,
) {
    let mut scan_row_visit_count = 0;
    let row = generation.row_by_id(requested).or_else(|| {
        let mut scan = generation.scan(ResourceManagementQuery::default());
        while let Some(row) = scan.next_row() {
            scan_row_visit_count = scan_row_visit_count.saturating_add(1);
            if let Ok(locator) = ResourceLocator::parse(row.primary_locator.as_ref()) {
                if ResourceId::from_locator(&locator) == requested {
                    return Some(row);
                }
            }
        }
        None
    });
    let Some(row) = row else {
        return (
            Err(UiTexturePrepareOutcome::UnresolvedIdentity),
            None,
            scan_row_visit_count,
        );
    };
    if row.kind != ResourceKind::Texture {
        return (
            Err(UiTexturePrepareOutcome::InvalidResourceKind),
            Some(row.id),
            scan_row_visit_count,
        );
    }
    let resolved = row.id;
    (Ok(row), Some(resolved), scan_row_visit_count)
}

impl ResourceStreamer {
    pub(in crate::graphics::scene::resources) fn prepare_ui_textures_for_frame(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        requested_ids: &UiTextureDependencies,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), crate::graphics::GraphicsError> {
        let asset_manager = self.asset_manager()?;
        let projection = asset_manager.resource_manager().projection_snapshot();
        let requested_ids = requested_ids.as_slice();
        let mut rows = Vec::with_capacity(requested_ids.len());
        let mut work = UiTexturePrepareWork::default();
        for &requested in requested_ids {
            rows.push(self.prepare_ui_texture_dependency(
                backend,
                texture_layout,
                asset_manager.as_ref(),
                projection.management(),
                &projection.readiness_identity(),
                requested,
                submission_transaction,
                &mut work,
            )?);
        }
        let receipt = UiTexturePrepareReceipt::new(
            self.next_ui_texture_prepare_epoch,
            projection.management_identity(),
            projection.readiness_identity(),
            rows,
        )
        .with_work(work);
        receipt.record_profile_counters();
        self.next_ui_texture_prepare_epoch =
            self.next_ui_texture_prepare_epoch.wrapping_add(1).max(1);
        self.last_ui_texture_prepare_receipt = Some(receipt);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_ui_texture_dependency(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        asset_manager: &ProjectAssetManager,
        management_generation: &Arc<ResourceManagementGeneration>,
        readiness_generation: &ResourceReadinessGenerationIdentity,
        requested: ResourceId,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
        work: &mut UiTexturePrepareWork,
    ) -> Result<UiTexturePrepareRow, crate::graphics::GraphicsError> {
        let (candidate, resolved, scan_row_visit_count) =
            resolve_ui_texture_candidate_with_work(management_generation, requested);
        work.resolution_scan_row_visit_count = work
            .resolution_scan_row_visit_count
            .saturating_add(scan_row_visit_count);
        let candidate = match candidate {
            Ok(candidate) => candidate,
            Err(outcome) => return Ok(prepare_row(requested, resolved, outcome, None)),
        };
        let resolved = candidate.id;
        if let Some((prepared_revision, prepared)) = self.texture_with_revision(resolved) {
            if prepared_revision == candidate.revision {
                work.prepared_reuse_count = work.prepared_reuse_count.saturating_add(1);
                let outcome = if is_ui_texture_descriptor(&prepared.descriptor) {
                    UiTexturePrepareOutcome::Ready
                } else {
                    UiTexturePrepareOutcome::InvalidDescriptor
                };
                return Ok(prepare_row(
                    requested,
                    Some(resolved),
                    outcome,
                    Some(prepared_revision),
                ));
            }
        }

        work.snapshot_load_count = work.snapshot_load_count.saturating_add(1);
        let texture = match asset_manager.load_texture_asset_snapshot(resolved) {
            Ok(texture) => texture,
            Err(_) => {
                let outcome = load_failure_outcome(readiness_generation, resolved);
                return Ok(prepare_row(
                    requested,
                    Some(resolved),
                    outcome,
                    self.texture_with_revision(resolved)
                        .map(|(revision, _)| revision),
                ));
            }
        };
        if texture.revision() != candidate.revision {
            return Ok(prepare_row(
                requested,
                Some(resolved),
                UiTexturePrepareOutcome::GenerationChanged,
                self.texture_with_revision(resolved)
                    .map(|(revision, _)| revision),
            ));
        }
        if !is_ui_texture_descriptor(&texture.render_image_descriptor()) {
            return Ok(prepare_row(
                requested,
                Some(resolved),
                UiTexturePrepareOutcome::InvalidDescriptor,
                self.texture_with_revision(resolved)
                    .map(|(revision, _)| revision),
            ));
        }
        let revision = texture.revision();
        work.upload_attempt_count = work.upload_attempt_count.saturating_add(1);
        let outcome = match self.ensure_texture_snapshot_for_frame(
            backend,
            texture_layout,
            resolved,
            texture,
            submission_transaction,
        ) {
            Ok(()) => UiTexturePrepareOutcome::Ready,
            Err(TextureSnapshotFramePrepareError::GpuArtifact) => {
                UiTexturePrepareOutcome::UploadFailed
            }
            Err(TextureSnapshotFramePrepareError::Submission(error)) => return Err(error),
        };
        Ok(prepare_row(
            requested,
            Some(resolved),
            outcome,
            self.texture_with_revision(resolved)
                .map(|(prepared_revision, _)| prepared_revision)
                .or((outcome == UiTexturePrepareOutcome::Ready).then_some(revision)),
        ))
    }
}

fn load_failure_outcome(
    readiness_generation: &ResourceReadinessGenerationIdentity,
    resolved: ResourceId,
) -> UiTexturePrepareOutcome {
    let state = readiness_generation
        .generation()
        .row_identity(resolved)
        .map(|row| row.row().typed_load_state::<TextureAsset>());
    if state == Some(ResourceReadinessState::Failed) {
        UiTexturePrepareOutcome::LoadFailed
    } else {
        UiTexturePrepareOutcome::NotReady
    }
}

const fn prepare_row(
    requested: ResourceId,
    resolved: Option<ResourceId>,
    outcome: UiTexturePrepareOutcome,
    prepared_revision: Option<u64>,
) -> UiTexturePrepareRow {
    UiTexturePrepareRow {
        requested,
        resolved,
        outcome,
        prepared_revision,
    }
}
