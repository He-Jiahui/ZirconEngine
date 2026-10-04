use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialReadinessReport, RenderMaterialValidationError,
};
use crate::core::resource::ResourceId;
use crate::graphics::types::GraphicsError;

use super::super::super::prepared::{
    PreparedMaterial, PreparedMaterialBundle, PreparedMaterialCandidateIdentity,
    RejectedPreparedMaterialCandidate,
};
use super::super::ResourceStreamer;
use super::material_readiness::material_readiness_allows_rendering;

impl ResourceStreamer {
    /// 先封存材质候选，保留旧 Published 代际供绘制；冷启动则暂用错误代理。
    pub(super) fn stage_material_candidate(
        &mut self,
        id: ResourceId,
        candidate: PreparedMaterialBundle,
    ) {
        if let Some(prepared) = self.materials.get_mut(&id) {
            prepared.staged_candidate = Some(candidate);
            prepared.staged_pipeline_failed = false;
            prepared.staged_pipeline_admission_cycle = Default::default();
            prepared.rejected_candidate = None;
            self.active_staged_material_ids.insert(id);
            crate::profile_counter!("render", "material_candidate_staged", 1);
            return;
        }
        self.active_staged_material_ids.insert(id);
        self.materials.insert(
            id,
            PreparedMaterial {
                published: None,
                previous_published: None,
                staged_candidate: Some(candidate),
                staged_pipeline_failed: false,
                staged_pipeline_admission_cycle: Default::default(),
                rejected_candidate: None,
            },
        );
        crate::profile_counter!("render", "material_candidate_staged", 1);
        crate::profile_counter!("render", "material_cold_error_proxy", 1);
    }

    /// 仅在管线准入周期完成后发布仍匹配资产及依赖修订的候选；
    /// draw proxy 随后按完整代际提供 runtime、uniform 与纹理绑定。
    pub(crate) fn publish_staged_material_candidate(&mut self, id: ResourceId) -> bool {
        let requested_revision = self.resource_revision(id).ok();
        let candidate_is_current = self
            .materials
            .get(&id)
            .and_then(|prepared| prepared.staged_candidate.as_ref())
            .is_some_and(|candidate| {
                self.prepared_material_bundle_cache_is_current(
                    candidate,
                    requested_revision,
                    candidate.texture_support,
                )
            });
        if !candidate_is_current {
            crate::profile_counter!("render", "material_candidate_publication_stale", 1);
            return false;
        }
        crate::profile_counter!("render", "material_candidate_publication_stale", 0);
        let Some(prepared) = self.materials.get_mut(&id) else {
            return false;
        };
        let Some(candidate) = prepared.staged_candidate.take() else {
            return false;
        };
        prepared.previous_published = prepared.published.take();
        prepared.published = Some(candidate);
        prepared.staged_pipeline_failed = false;
        prepared.staged_pipeline_admission_cycle = Default::default();
        prepared.rejected_candidate = None;
        self.active_staged_material_ids.remove(&id);
        true
    }

    pub(crate) fn reject_staged_material_pipeline_candidate(
        &mut self,
        id: ResourceId,
        path: impl Into<String>,
        diagnostic: impl Into<String>,
    ) -> bool {
        let Some(prepared) = self.materials.get_mut(&id) else {
            return false;
        };
        let Some(candidate) = prepared.staged_candidate.as_ref() else {
            return false;
        };
        let identity = candidate.candidate_identity();
        let mut readiness_report = candidate.runtime.readiness_report.clone();
        readiness_report.push_validation_error_once(
            RenderMaterialValidationError::ShaderReadinessDiagnostic {
                source: RenderMaterialDiagnosticSource::ShaderReadiness,
                path: path.into(),
                diagnostic: diagnostic.into(),
            },
        );
        prepared.rejected_candidate = Some(RejectedPreparedMaterialCandidate {
            identity: Some(identity),
            readiness_report,
        });
        prepared.staged_pipeline_failed = true;
        prepared.staged_pipeline_admission_cycle = Default::default();
        self.active_staged_material_ids.remove(&id);
        crate::profile_counter!("render", "material_candidate_pipeline_failed", 1);
        true
    }

    pub(crate) fn record_staged_material_pipeline_admission(
        &mut self,
        id: ResourceId,
        deferred: bool,
    ) -> bool {
        let Some(prepared) = self.materials.get_mut(&id) else {
            return false;
        };
        if prepared.staged_candidate.is_none() || prepared.staged_pipeline_failed {
            return false;
        }
        prepared.staged_pipeline_admission_cycle.record(deferred);
        true
    }

    pub(crate) fn reset_staged_material_pipeline_admission_cycle(
        &mut self,
        id: ResourceId,
    ) -> bool {
        let Some(prepared) = self.materials.get_mut(&id) else {
            return false;
        };
        if prepared.staged_candidate.is_none() || prepared.staged_pipeline_failed {
            return false;
        }
        prepared.staged_pipeline_admission_cycle = Default::default();
        true
    }

    /// Completes the current viewport admission cycle. `None` means the
    /// candidate was not referenced by any camera in this viewport.
    pub(crate) fn finish_staged_material_pipeline_admission_cycle(
        &mut self,
        id: ResourceId,
    ) -> Option<bool> {
        let prepared = self.materials.get_mut(&id)?;
        if prepared.staged_candidate.is_none() || prepared.staged_pipeline_failed {
            prepared.staged_pipeline_admission_cycle = Default::default();
            return None;
        }
        prepared.staged_pipeline_admission_cycle.finish()
    }

    pub(crate) fn park_unobserved_staged_material_candidate(&mut self, id: ResourceId) -> bool {
        let Some(prepared) = self.materials.get_mut(&id) else {
            return false;
        };
        if prepared.staged_candidate.is_none() || prepared.staged_pipeline_failed {
            return false;
        }
        prepared.staged_pipeline_admission_cycle = Default::default();
        self.active_staged_material_ids.remove(&id)
    }

    pub(super) fn retain_last_good_material_candidate(
        &mut self,
        id: ResourceId,
        identity: Option<PreparedMaterialCandidateIdentity>,
        readiness_report: RenderMaterialReadinessReport,
    ) -> Result<(), RenderMaterialReadinessReport> {
        let Some(published) = self.materials.get_mut(&id) else {
            return Err(readiness_report);
        };
        if !published.published.as_ref().is_some_and(|bundle| {
            material_readiness_allows_rendering(&bundle.runtime.readiness_report)
        }) {
            return Err(readiness_report);
        }
        published.rejected_candidate = Some(RejectedPreparedMaterialCandidate {
            identity,
            readiness_report,
        });
        published.staged_candidate = None;
        published.staged_pipeline_failed = false;
        published.staged_pipeline_admission_cycle = Default::default();
        self.active_staged_material_ids.remove(&id);
        crate::profile_counter!("render", "material_last_good_rejection", 1);
        Ok(())
    }

    pub(super) fn retain_last_good_material_after_candidate_failure(
        &mut self,
        id: ResourceId,
        mut readiness_report: RenderMaterialReadinessReport,
        path: impl Into<String>,
        error: &GraphicsError,
    ) -> Result<(), RenderMaterialReadinessReport> {
        readiness_report.push_validation_error_once(
            RenderMaterialValidationError::ShaderReadinessDiagnostic {
                source: RenderMaterialDiagnosticSource::DependencyResolution,
                path: path.into(),
                diagnostic: error.to_string(),
            },
        );
        // Residency, I/O, queue, receipt, device, and channel failures may
        // recover without changing the asset revision. Keep last-good visible,
        // but do not suppress the next preparation attempt.
        self.retain_last_good_material_candidate(id, None, readiness_report)
    }
}

#[cfg(test)]
#[path = "tests/candidate_publication.rs"]
mod tests;
