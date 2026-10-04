use std::collections::{BTreeMap, BTreeSet};
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

use super::{is_ui_texture_descriptor, UiTextureDependencies};

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
    binding_product_generation: u64,
    dependency_generation: u64,
    management_generation: ResourceManagementGenerationIdentity,
    readiness_generation: ResourceReadinessGenerationIdentity,
    rows: BTreeMap<ResourceId, UiTexturePrepareRow>,
    frame_retry_ids: BTreeSet<ResourceId>,
    readiness_retry_ids: BTreeSet<ResourceId>,
    summary: UiTexturePrepareSummary,
    work: UiTexturePrepareWork,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct UiTexturePrepareWork {
    resolution_scan_row_visit_count: usize,
    snapshot_load_count: usize,
    prepared_reuse_count: usize,
    upload_attempt_count: usize,
    dependency_prepare_visit_count: usize,
    retained_row_count: usize,
}

impl UiTexturePrepareReceipt {
    pub(super) fn new(
        frame_prepare_epoch: u64,
        management_generation: ResourceManagementGenerationIdentity,
        readiness_generation: ResourceReadinessGenerationIdentity,
        mut rows: Vec<UiTexturePrepareRow>,
    ) -> Self {
        rows.sort_unstable_by_key(|row| row.requested);
        debug_assert!(rows
            .windows(2)
            .all(|pair| pair[0].requested != pair[1].requested));
        let rows = rows
            .into_iter()
            .map(|row| (row.requested, row))
            .collect::<BTreeMap<_, _>>();
        let summary = UiTexturePrepareSummary::from_rows(rows.values());
        let (frame_retry_ids, readiness_retry_ids) = retry_indexes(rows.values());
        Self {
            frame_prepare_epoch,
            binding_product_generation: frame_prepare_epoch,
            dependency_generation: 0,
            management_generation,
            readiness_generation,
            rows,
            frame_retry_ids,
            readiness_retry_ids,
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

    pub(in crate::graphics::scene) const fn binding_product_generation(&self) -> u64 {
        self.binding_product_generation
    }

    // 每帧准备 epoch 可推进，绑定产物代次仅在 row 改变时推进；完全相同的结果复用旧代次，避免触发无效绑定重建。
    pub(super) fn reuse_equivalent_binding_product_generation(&mut self, previous: &Self) {
        if self.rows == previous.rows {
            self.binding_product_generation = previous.binding_product_generation;
        }
    }

    fn with_dependency_generation(mut self, dependency_generation: u64) -> Self {
        self.dependency_generation = dependency_generation;
        self
    }

    fn begin_retained_frame(
        &mut self,
        frame_prepare_epoch: u64,
        dependency_generation: u64,
        management_generation: ResourceManagementGenerationIdentity,
        readiness_generation: ResourceReadinessGenerationIdentity,
    ) {
        self.frame_prepare_epoch = frame_prepare_epoch;
        self.dependency_generation = dependency_generation;
        self.management_generation = management_generation;
        self.readiness_generation = readiness_generation;
        self.work = UiTexturePrepareWork::default();
    }

    fn remove_requested(&mut self, requested: ResourceId) -> bool {
        let Some(previous) = self.rows.remove(&requested) else {
            return false;
        };
        self.summary.remove_row(previous);
        self.frame_retry_ids.remove(&requested);
        self.readiness_retry_ids.remove(&requested);
        true
    }

    fn upsert_row(&mut self, row: UiTexturePrepareRow) -> bool {
        if self.rows.get(&row.requested) == Some(&row) {
            return false;
        }
        if let Some(previous) = self.rows.insert(row.requested, row) {
            self.summary.remove_row(previous);
        }
        self.summary.add_row(row);
        self.frame_retry_ids.remove(&row.requested);
        self.readiness_retry_ids.remove(&row.requested);
        match retry_policy(row.outcome) {
            UiTextureRetryPolicy::EveryFrame => {
                self.frame_retry_ids.insert(row.requested);
            }
            UiTextureRetryPolicy::ReadinessGeneration => {
                self.readiness_retry_ids.insert(row.requested);
            }
            UiTextureRetryPolicy::ManagementGeneration => {}
        }
        true
    }

    fn mark_binding_product_changed(&mut self) {
        self.binding_product_generation = self.frame_prepare_epoch;
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
        let row = self.rows.get(&requested)?;
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
                (
                    "ui.ui_texture_prepare.dependency_prepare_visit_count",
                    work.dependency_prepare_visit_count as f64,
                ),
                (
                    "ui.ui_texture_prepare.retained_row_count",
                    work.retained_row_count as f64,
                ),
            ],
        );
    }
}

impl UiTexturePrepareSummary {
    fn from_rows<'a>(rows: impl IntoIterator<Item = &'a UiTexturePrepareRow>) -> Self {
        let mut summary = Self::default();
        for row in rows {
            summary.add_row(*row);
        }
        summary
    }

    fn add_row(&mut self, row: UiTexturePrepareRow) {
        self.requested_count = self.requested_count.saturating_add(1);
        let outcome_count = self.outcome_count_mut(row.outcome);
        *outcome_count = outcome_count.saturating_add(1);
    }

    fn remove_row(&mut self, row: UiTexturePrepareRow) {
        self.requested_count = self.requested_count.saturating_sub(1);
        let outcome_count = self.outcome_count_mut(row.outcome);
        *outcome_count = outcome_count.saturating_sub(1);
    }

    fn outcome_count_mut(&mut self, outcome: UiTexturePrepareOutcome) -> &mut usize {
        match outcome {
            UiTexturePrepareOutcome::UnresolvedIdentity => &mut self.unresolved_count,
            UiTexturePrepareOutcome::NotReady => &mut self.not_ready_count,
            UiTexturePrepareOutcome::LoadFailed => &mut self.load_failed_count,
            UiTexturePrepareOutcome::InvalidResourceKind => &mut self.invalid_resource_kind_count,
            UiTexturePrepareOutcome::InvalidDescriptor => &mut self.invalid_descriptor_count,
            UiTexturePrepareOutcome::GenerationChanged => &mut self.generation_changed_count,
            UiTexturePrepareOutcome::UploadFailed => &mut self.upload_failed_count,
            UiTexturePrepareOutcome::Ready => &mut self.ready_count,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiTextureRetryPolicy {
    EveryFrame,
    ReadinessGeneration,
    ManagementGeneration,
}

// 上传失败与代际变化每帧重试；加载未就绪或失败等待 readiness 更新，身份及描述符结果随 management 更新重新判断。
const fn retry_policy(outcome: UiTexturePrepareOutcome) -> UiTextureRetryPolicy {
    match outcome {
        UiTexturePrepareOutcome::UploadFailed | UiTexturePrepareOutcome::GenerationChanged => {
            UiTextureRetryPolicy::EveryFrame
        }
        UiTexturePrepareOutcome::NotReady | UiTexturePrepareOutcome::LoadFailed => {
            UiTextureRetryPolicy::ReadinessGeneration
        }
        UiTexturePrepareOutcome::UnresolvedIdentity
        | UiTexturePrepareOutcome::InvalidResourceKind
        | UiTexturePrepareOutcome::InvalidDescriptor
        | UiTexturePrepareOutcome::Ready => UiTextureRetryPolicy::ManagementGeneration,
    }
}

fn retry_indexes<'a>(
    rows: impl IntoIterator<Item = &'a UiTexturePrepareRow>,
) -> (BTreeSet<ResourceId>, BTreeSet<ResourceId>) {
    let mut frame_retry_ids = BTreeSet::new();
    let mut readiness_retry_ids = BTreeSet::new();
    for row in rows {
        match retry_policy(row.outcome) {
            UiTextureRetryPolicy::EveryFrame => {
                frame_retry_ids.insert(row.requested);
            }
            UiTextureRetryPolicy::ReadinessGeneration => {
                readiness_retry_ids.insert(row.requested);
            }
            UiTextureRetryPolicy::ManagementGeneration => {}
        }
    }
    (frame_retry_ids, readiness_retry_ids)
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
    let mut scan_row_visit_count: usize = 0;
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
        let mut work = UiTexturePrepareWork::default();
        let frame_prepare_epoch = self.next_ui_texture_prepare_epoch;
        let previous = self.last_ui_texture_prepare_receipt.take();
        let exact_dependency_product = previous
            .as_ref()
            .is_some_and(|receipt| receipt.dependency_generation == requested_ids.generation());
        let journal = requested_ids.change_journal();
        let journal_applies = previous.as_ref().is_some_and(|receipt| {
            !journal.is_full_rebuild()
                && journal.base_generation() == Some(receipt.dependency_generation)
        });

        let receipt = if exact_dependency_product || journal_applies {
            let mut receipt = previous
                .expect("a retained UI texture dependency generation must retain its receipt");
            let management_changed =
                receipt.management_generation != projection.management_identity();
            let readiness_changed = receipt.readiness_generation != projection.readiness_identity();
            let mut dirty_ids = BTreeSet::new();
            let mut rows_changed = false;
            if journal_applies {
                for &removed in journal.removed_ids() {
                    rows_changed |= receipt.remove_requested(removed);
                }
                dirty_ids.extend(journal.added_ids().iter().copied());
            }
            if management_changed {
                dirty_ids.extend(requested_ids.as_slice().iter().copied());
            } else {
                dirty_ids.extend(receipt.frame_retry_ids.iter().copied());
                if readiness_changed {
                    dirty_ids.extend(receipt.readiness_retry_ids.iter().copied());
                }
            }
            receipt.begin_retained_frame(
                frame_prepare_epoch,
                requested_ids.generation(),
                projection.management_identity(),
                projection.readiness_identity(),
            );
            work.dependency_prepare_visit_count = dirty_ids.len();
            let existing_dirty_row_count = dirty_ids
                .iter()
                .filter(|requested| receipt.rows.contains_key(requested))
                .count();
            work.retained_row_count = receipt.rows.len().saturating_sub(existing_dirty_row_count);
            for requested in dirty_ids {
                let row = match self.prepare_ui_texture_dependency(
                    backend,
                    texture_layout,
                    asset_manager.as_ref(),
                    projection.management(),
                    &projection.readiness_identity(),
                    requested,
                    submission_transaction,
                    &mut work,
                ) {
                    Ok(row) => row,
                    Err(error) => {
                        self.last_ui_texture_prepare_receipt = None;
                        return Err(error);
                    }
                };
                rows_changed |= receipt.upsert_row(row);
            }
            if rows_changed {
                receipt.mark_binding_product_changed();
            }
            receipt.with_work(work)
        } else {
            let mut rows = Vec::with_capacity(requested_ids.as_slice().len());
            work.dependency_prepare_visit_count = requested_ids.as_slice().len();
            for &requested in requested_ids.as_slice() {
                let row = match self.prepare_ui_texture_dependency(
                    backend,
                    texture_layout,
                    asset_manager.as_ref(),
                    projection.management(),
                    &projection.readiness_identity(),
                    requested,
                    submission_transaction,
                    &mut work,
                ) {
                    Ok(row) => row,
                    Err(error) => {
                        self.last_ui_texture_prepare_receipt = None;
                        return Err(error);
                    }
                };
                rows.push(row);
            }
            UiTexturePrepareReceipt::new(
                frame_prepare_epoch,
                projection.management_identity(),
                projection.readiness_identity(),
                rows,
            )
            .with_dependency_generation(requested_ids.generation())
            .with_work(work)
        };
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

#[cfg(test)]
#[path = "tests/prepare_receipt.rs"]
mod tests;
