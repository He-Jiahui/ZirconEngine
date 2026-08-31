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
mod tests {
    #[test]
    fn dependency_execution_failures_keep_last_good_without_suppressing_retry() {
        let source = include_str!("candidate_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("candidate publication test boundary");
        let retention = source
            .split("fn retain_last_good_material_after_candidate_failure")
            .nth(1)
            .expect("dependency failure retention");

        assert!(retention.contains("retain_last_good_material_candidate(id, None"));
        assert!(!retention.contains("cache_material_candidate_failure"));
    }

    #[test]
    fn terminal_pipeline_rejection_keeps_the_exact_staged_identity_for_cache_suppression() {
        let source = include_str!("candidate_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("candidate publication test boundary");
        let rejection = source
            .split("fn reject_staged_material_pipeline_candidate(")
            .nth(1)
            .and_then(|source| {
                source
                    .split("fn retain_last_good_material_candidate(")
                    .next()
            })
            .expect("pipeline rejection function");

        assert!(rejection.contains("prepared.staged_candidate.as_ref()"));
        assert!(!rejection.contains("prepared.staged_candidate.take()"));
        assert!(rejection.contains("prepared.staged_pipeline_failed = true"));
        assert!(rejection.contains("self.active_staged_material_ids.remove(&id)"));
        assert!(source.contains("self.active_staged_material_ids.insert(id)"));
    }

    #[test]
    fn cold_materials_remain_staged_until_pipeline_publication() {
        let source = include_str!("candidate_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("candidate publication test boundary");
        let cold_insert = source
            .split("self.materials.insert(")
            .nth(1)
            .and_then(|source| source.split("pub(crate) fn publish_staged").next())
            .expect("cold candidate insertion");

        assert!(cold_insert.contains("published: None"));
        assert!(cold_insert.contains("staged_candidate: Some(candidate)"));
        assert!(cold_insert.contains("self.active_staged_material_ids.insert(id)"));
        assert!(source.contains("prepared.published = Some(candidate)"));
        assert!(source.contains("prepared.previous_published = prepared.published.take()"));
    }

    #[test]
    fn candidate_publication_tracks_the_complete_viewport_cycle() {
        let source = include_str!("candidate_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("candidate publication test boundary");

        assert!(source.contains("record_staged_material_pipeline_admission"));
        assert!(source.contains("reset_staged_material_pipeline_admission_cycle"));
        assert!(source.contains("finish_staged_material_pipeline_admission_cycle"));
        assert!(source.contains("park_unobserved_staged_material_candidate"));
        assert!(source.contains("staged_pipeline_admission_cycle.record(deferred)"));
        assert!(source.contains("staged_pipeline_admission_cycle.finish()"));
    }

    #[test]
    fn parked_candidate_reactivates_only_when_material_preparation_touches_it_again() {
        let publication_source = include_str!("candidate_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("candidate publication test boundary");
        let ensure_source = include_str!("../resource_streamer_ensure_material.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material preparation test boundary");

        assert!(publication_source.contains("park_unobserved_staged_material_candidate"));
        assert!(publication_source.contains("self.active_staged_material_ids.remove(&id)"));
        assert!(ensure_source.contains("current_slot == PreparedMaterialCacheSlot::Staged"));
        assert!(ensure_source.contains("self.active_staged_material_ids.insert(id)"));
        assert!(ensure_source.contains("material_candidate_reactivated"));
    }

    #[test]
    fn stale_candidate_is_rechecked_before_draw_visible_publication() {
        let source = include_str!("candidate_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("candidate publication test boundary");
        let publication = source
            .split("fn publish_staged_material_candidate")
            .nth(1)
            .expect("candidate publication function");

        assert!(publication.contains("material_candidate_publication_stale"));
        assert!(publication.contains("prepared_material_bundle_cache_is_current"));
        assert!(publication.contains("staged_candidate.as_ref()"));
        assert!(
            publication
                .find("prepared_material_bundle_cache_is_current")
                .unwrap()
                < publication.find("staged_candidate.take()").unwrap()
        );
    }

    #[test]
    fn publication_does_not_create_a_permanent_context_admission_owner() {
        let source = include_str!("candidate_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("candidate publication test boundary");

        assert!(!source.contains("context_admission_material_ids"));
    }

    #[test]
    fn unchanged_non_pipeline_rejection_is_suppressed_before_material_rebuild() {
        let prepared_source = include_str!("../../prepared/prepared_material.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("prepared material test boundary");
        let ensure_source = include_str!("../resource_streamer_ensure_material.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material preparation test boundary");
        let cache_probe = ensure_source
            .split("let current_slot =")
            .nth(1)
            .and_then(|source| source.split("if let Some(current_slot)").next())
            .expect("material cache probe");
        let cache_hit = ensure_source
            .split("if let Some(current_slot)")
            .nth(1)
            .and_then(|source| source.split("material_prepare_rebuild").next())
            .expect("material cache-hit handling");

        assert!(prepared_source.contains("PreparedMaterialCandidateIdentity"));
        assert!(prepared_source.contains("identity: Option<PreparedMaterialCandidateIdentity>"));
        assert!(cache_probe.contains("PreparedMaterialCacheSlot::RejectedCandidate"));
        assert!(cache_probe.contains("prepared_material_candidate_cache_is_current"));
        assert!(cache_hit.contains("PreparedMaterialCacheSlot::RejectedCandidate"));
        assert!(cache_hit.contains("return Ok(())"));
    }
}
