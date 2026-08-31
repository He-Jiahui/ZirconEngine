use crate::graphics::scene::resources::ResourceStreamer;

use super::mesh_pipeline_cache::MaterialPipelinePublicationAdmission;
use super::{MaterialPipelineRequirementCensus, MeshPipelineCache};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MaterialPipelinePublicationStats {
    pub(crate) candidate_count: usize,
    pub(crate) published_count: usize,
    pub(crate) deferred_count: usize,
    pub(crate) failed_count: usize,
    pub(crate) unobserved_count: usize,
    pub(crate) requirement_count: usize,
    pub(crate) ready_requirement_count: usize,
}

pub(crate) fn coordinate_material_pipeline_publications(
    device: &wgpu::Device,
    streamer: &mut ResourceStreamer,
    mesh_pipelines: &mut MeshPipelineCache,
    mut census: MaterialPipelineRequirementCensus,
    publication_cycle_start: bool,
    publication_boundary: bool,
) -> MaterialPipelinePublicationStats {
    let mut candidate_ids = streamer.staged_material_candidate_ids().collect::<Vec<_>>();
    candidate_ids.sort_unstable();
    let mut stats = MaterialPipelinePublicationStats {
        candidate_count: candidate_ids.len(),
        ..MaterialPipelinePublicationStats::default()
    };

    for material_id in candidate_ids {
        if publication_cycle_start {
            streamer.reset_staged_material_pipeline_admission_cycle(material_id);
        }
        let generation = streamer
            .staged_material_draw_generation(&material_id)
            .expect("active staged material ID must retain its draw generation");
        if let Some(requirements) = census.remove(material_id, generation) {
            let admission = mesh_pipelines.ensure_material_pipeline_requirements_for_generation(
                device,
                streamer,
                material_id,
                generation,
                &requirements,
            );
            stats.requirement_count = stats
                .requirement_count
                .saturating_add(admission.requirement_count());
            stats.ready_requirement_count = stats
                .ready_requirement_count
                .saturating_add(admission.ready_count());
            match admission {
                MaterialPipelinePublicationAdmission::Ready { .. } => {
                    streamer.record_staged_material_pipeline_admission(material_id, false);
                }
                MaterialPipelinePublicationAdmission::Deferred { .. } => {
                    streamer.record_staged_material_pipeline_admission(material_id, true);
                    if !publication_boundary {
                        stats.deferred_count = stats.deferred_count.saturating_add(1);
                    }
                }
                MaterialPipelinePublicationAdmission::Failed {
                    requirement,
                    unavailable,
                    ..
                } => {
                    stats.failed_count = stats.failed_count.saturating_add(1);
                    streamer.reject_staged_material_pipeline_candidate(
                        material_id,
                        format!("pipeline_requirements.{:?}", requirement.target()),
                        format!(
                            "variant {} failed pipeline admission: {}",
                            requirement.variant_id().value(),
                            unavailable.reason().label()
                        ),
                    );
                    continue;
                }
            }
        }

        if !publication_boundary {
            continue;
        }
        match streamer.finish_staged_material_pipeline_admission_cycle(material_id) {
            Some(true) => {
                if streamer.publish_staged_material_candidate(material_id) {
                    stats.published_count = stats.published_count.saturating_add(1);
                }
            }
            Some(false) => {
                stats.deferred_count = stats.deferred_count.saturating_add(1);
            }
            None => {
                streamer.park_unobserved_staged_material_candidate(material_id);
                stats.unobserved_count = stats.unobserved_count.saturating_add(1);
            }
        }
    }

    crate::profile_counter!(
        "render",
        "material_pipeline_candidate_count",
        stats.candidate_count
    );
    crate::profile_counter!(
        "render",
        "material_pipeline_candidate_published",
        stats.published_count
    );
    crate::profile_counter!(
        "render",
        "material_pipeline_candidate_deferred",
        stats.deferred_count
    );
    crate::profile_counter!(
        "render",
        "material_pipeline_candidate_failed",
        stats.failed_count
    );
    crate::profile_counter!(
        "render",
        "material_pipeline_candidate_unobserved",
        stats.unobserved_count
    );
    stats
}

#[cfg(test)]
mod tests {
    #[test]
    fn coordinator_source_keeps_publication_behind_all_ready_admission() {
        let source = include_str!("material_pipeline_publication_coordinator.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("publication coordinator test boundary");
        let admission = source
            .split("match admission {")
            .nth(1)
            .expect("publication admission match");

        assert!(admission.contains("MaterialPipelinePublicationAdmission::Ready"));
        assert!(admission.contains("publish_staged_material_candidate"));
        assert!(admission.contains("MaterialPipelinePublicationAdmission::Deferred"));
        assert!(admission.contains("deferred_count"));
        assert!(admission.contains("MaterialPipelinePublicationAdmission::Failed"));
        assert!(admission.contains("reject_staged_material_pipeline_candidate"));
        assert!(source.contains("if !publication_boundary"));
        assert!(source.contains("if publication_cycle_start"));
        assert!(source.contains("reset_staged_material_pipeline_admission_cycle"));
        assert!(source.contains("finish_staged_material_pipeline_admission_cycle"));
        assert!(source.contains("park_unobserved_staged_material_candidate"));
        assert!(source.contains("staged_material_draw_generation"));
        assert!(source.contains("ensure_material_pipeline_requirements_for_generation"));
        assert!(!source.contains("let Some(requirements) = census.remove(&material_id) else"));
    }
}
