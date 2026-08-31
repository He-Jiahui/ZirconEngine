use crate::core::framework::render::ShaderQualityTier;
use crate::graphics::scene::resources::{MaterialDrawGenerationSelection, ResourceStreamer};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::{
    MaterialPipelinePublicationAdmission, MeshPipelineCache,
};
use crate::graphics::types::GraphicsError;

use super::material_draw_selection::MaterialDrawSelection;
use super::material_pipeline_requirements::{
    MaterialPipelineFeatureSet, MaterialPipelineRequirementCensus,
    collect_error_proxy_context_pipeline_requirements,
    collect_previous_context_pipeline_requirements,
};
use super::pending_mesh_draw::PendingMeshDraw;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct MaterialContextAdmissionStats {
    pub(super) tracked_material_count: usize,
    pub(super) scanned_draw_count: usize,
    pub(super) candidate_count: usize,
    pub(super) current_ready_count: usize,
    pub(super) previous_selected_count: usize,
    pub(super) error_proxy_selected_count: usize,
    pub(super) deferred_count: usize,
    pub(super) failed_count: usize,
    pub(super) requirement_count: usize,
    pub(super) ready_requirement_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MaterialGenerationRequirementCacheStats {
    hit_material_count: usize,
    observed_requirement_count: usize,
    hit_requirement_count: usize,
}

impl MaterialGenerationRequirementCacheStats {
    fn miss_requirement_count(self) -> usize {
        self.observed_requirement_count
            .saturating_sub(self.hit_requirement_count)
    }
}

pub(super) fn select_material_generations_for_context(
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    mesh_pipelines: &mut MeshPipelineCache,
    pending_draws: &[PendingMeshDraw],
    mut current_census: MaterialPipelineRequirementCensus,
    features: MaterialPipelineFeatureSet,
    shader_quality: ShaderQualityTier,
    volumetric_fog_enabled: bool,
) -> Result<(MaterialDrawSelection, MaterialContextAdmissionStats), GraphicsError> {
    crate::profile_scope!("render", "material", "context_admission");
    let tracked_material_count = current_census.len();
    let scanned_draw_count = pending_draws.len();
    let generation_cache_stats =
        retain_material_pipeline_requirement_misses(streamer, mesh_pipelines, &mut current_census);
    let mut selection = MaterialDrawSelection::default();
    let mut stats = MaterialContextAdmissionStats {
        tracked_material_count,
        scanned_draw_count,
        candidate_count: current_census.len(),
        current_ready_count: generation_cache_stats.hit_material_count,
        ..MaterialContextAdmissionStats::default()
    };

    for (material_id, admission) in
        admit_census(device, streamer, mesh_pipelines, current_census, &mut stats)
    {
        let generation_selection = generation_selection_for_current(
            matches!(
                admission,
                MaterialPipelinePublicationAdmission::Ready { .. }
            ),
            streamer
                .material_draw_proxy(
                    &material_id,
                    MaterialDrawGenerationSelection::PreviousPublished,
                )
                .runtime()
                .is_some(),
        );
        match generation_selection {
            MaterialDrawGenerationSelection::Published => {
                stats.current_ready_count = stats.current_ready_count.saturating_add(1);
            }
            MaterialDrawGenerationSelection::PreviousPublished => {
                selection.select(material_id, generation_selection);
                stats.previous_selected_count = stats.previous_selected_count.saturating_add(1);
            }
            MaterialDrawGenerationSelection::ErrorProxy => {
                selection.select(material_id, generation_selection);
                stats.error_proxy_selected_count =
                    stats.error_proxy_selected_count.saturating_add(1);
            }
        }
    }

    let previous_census = collect_previous_context_pipeline_requirements(
        pending_draws,
        streamer,
        &selection,
        features,
        shader_quality,
        volumetric_fog_enabled,
    );
    for (material_id, admission) in admit_census(
        device,
        streamer,
        mesh_pipelines,
        previous_census,
        &mut stats,
    ) {
        if matches!(
            admission,
            MaterialPipelinePublicationAdmission::Ready { .. }
        ) {
            continue;
        }
        selection.select(material_id, MaterialDrawGenerationSelection::ErrorProxy);
        stats.previous_selected_count = stats.previous_selected_count.saturating_sub(1);
        stats.error_proxy_selected_count = stats.error_proxy_selected_count.saturating_add(1);
    }

    let error_proxy_requirements = collect_error_proxy_context_pipeline_requirements(
        pending_draws,
        streamer,
        &selection,
        features,
        shader_quality,
        volumetric_fog_enabled,
    );
    let error_proxy_requirement_count = error_proxy_requirements.len();
    mesh_pipelines
        .ensure_error_proxy_pipeline_requirements(device, streamer, &error_proxy_requirements)
        .map_err(|message| {
            GraphicsError::Asset(format!("error material is unavailable: {message}"))
        })?;
    stats.requirement_count = stats
        .requirement_count
        .saturating_add(error_proxy_requirement_count);
    stats.ready_requirement_count = stats
        .ready_requirement_count
        .saturating_add(error_proxy_requirement_count);

    crate::profile_counter!(
        "render",
        "material_context_admission_tracked_material_count",
        stats.tracked_material_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_scanned_draw_count",
        stats.scanned_draw_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_candidate_count",
        stats.candidate_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_generation_cache_hit_count",
        generation_cache_stats.hit_material_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_generation_cache_miss_count",
        stats.candidate_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_observed_requirement_count",
        generation_cache_stats.observed_requirement_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_generation_cache_hit_requirement_count",
        generation_cache_stats.hit_requirement_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_generation_cache_miss_requirement_count",
        generation_cache_stats.miss_requirement_count()
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_previous_selected",
        stats.previous_selected_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_current_ready",
        stats.current_ready_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_error_proxy_selected",
        stats.error_proxy_selected_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_requirement_count",
        stats.requirement_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_ready_requirement_count",
        stats.ready_requirement_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_deferred_count",
        stats.deferred_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_failed_count",
        stats.failed_count
    );

    Ok((selection, stats))
}

fn retain_material_pipeline_requirement_misses(
    streamer: &ResourceStreamer,
    mesh_pipelines: &mut MeshPipelineCache,
    census: &mut MaterialPipelineRequirementCensus,
) -> MaterialGenerationRequirementCacheStats {
    let mut stats = MaterialGenerationRequirementCacheStats::default();
    census.retain(|material_id, generation, requirements| {
        stats.observed_requirement_count = stats
            .observed_requirement_count
            .saturating_add(requirements.len());
        let ready = mesh_pipelines.material_pipeline_requirements_are_ready_for_generation(
            streamer,
            *material_id,
            generation,
            requirements,
        );
        if ready {
            stats.hit_material_count = stats.hit_material_count.saturating_add(1);
            stats.hit_requirement_count = stats
                .hit_requirement_count
                .saturating_add(requirements.len());
        }
        !ready
    });
    stats
}

fn generation_selection_for_current(
    current_ready: bool,
    previous_published_exists: bool,
) -> MaterialDrawGenerationSelection {
    if current_ready {
        MaterialDrawGenerationSelection::Published
    } else if previous_published_exists {
        MaterialDrawGenerationSelection::PreviousPublished
    } else {
        MaterialDrawGenerationSelection::ErrorProxy
    }
}

fn admit_census(
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    mesh_pipelines: &mut MeshPipelineCache,
    census: MaterialPipelineRequirementCensus,
    stats: &mut MaterialContextAdmissionStats,
) -> Vec<(
    crate::core::resource::ResourceId,
    MaterialPipelinePublicationAdmission,
)> {
    let mut rows = census.into_requirements().collect::<Vec<_>>();
    rows.sort_unstable_by_key(|(material_id, generation, _)| (*material_id, *generation));
    rows.into_iter()
        .map(|(material_id, generation, requirements)| {
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
                MaterialPipelinePublicationAdmission::Ready { .. } => {}
                MaterialPipelinePublicationAdmission::Deferred { .. } => {
                    stats.deferred_count = stats.deferred_count.saturating_add(1);
                }
                MaterialPipelinePublicationAdmission::Failed { .. } => {
                    stats.failed_count = stats.failed_count.saturating_add(1);
                }
            }
            (material_id, admission)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::graphics::scene::resources::MaterialDrawGenerationSelection;

    use super::{MaterialGenerationRequirementCacheStats, generation_selection_for_current};

    #[test]
    fn generation_requirement_cache_miss_count_is_the_observed_remainder() {
        let stats = MaterialGenerationRequirementCacheStats {
            hit_material_count: 2,
            observed_requirement_count: 11,
            hit_requirement_count: 7,
        };

        assert_eq!(stats.miss_requirement_count(), 4);
    }

    #[test]
    fn current_context_admission_uses_atomic_previous_or_error_fallback() {
        assert_eq!(
            generation_selection_for_current(true, true),
            MaterialDrawGenerationSelection::Published
        );
        assert_eq!(
            generation_selection_for_current(false, true),
            MaterialDrawGenerationSelection::PreviousPublished
        );
        assert_eq!(
            generation_selection_for_current(false, false),
            MaterialDrawGenerationSelection::ErrorProxy
        );
    }

    #[test]
    fn context_admission_selects_complete_previous_or_error_material_proxies() {
        let source = include_str!("material_context_admission.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material context admission test boundary");

        assert!(source.contains("MaterialPipelinePublicationAdmission::Deferred"));
        assert!(source.contains("MaterialPipelinePublicationAdmission::Failed"));
        assert!(source.contains("MaterialDrawGenerationSelection::PreviousPublished"));
        assert!(source.contains("MaterialDrawGenerationSelection::ErrorProxy"));
        assert!(source.contains("collect_previous_context_pipeline_requirements"));
        assert!(source.contains("ensure_material_pipeline_requirements_for_generation"));
        assert!(source.contains("draw_generation()"));
        assert!(source.contains("\"material\", \"context_admission\""));
    }

    #[test]
    fn context_admission_profiles_fused_census_and_generation_cache_scale() {
        let source = include_str!("material_context_admission.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material context admission test boundary");

        assert!(!source.contains("context_admission_material_count"));
        assert!(source.contains("material_context_admission_tracked_material_count"));
        assert!(source.contains("material_context_admission_scanned_draw_count"));
        assert!(source.contains("material_context_admission_candidate_count"));
        assert!(source.contains("material_context_admission_generation_cache_hit_count"));
        assert!(source.contains("material_context_admission_generation_cache_miss_count"));
        assert!(source.contains("material_context_admission_observed_requirement_count"));
        assert!(
            source.contains("material_context_admission_generation_cache_hit_requirement_count")
        );
        assert!(
            source.contains("material_context_admission_generation_cache_miss_requirement_count")
        );
    }

    #[test]
    fn context_admission_consumes_fused_generation_ledger_misses() {
        let source = include_str!("material_context_admission.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material context admission test boundary");
        let selection = source
            .split("fn select_material_generations_for_context(")
            .nth(1)
            .expect("context material selection");

        assert!(selection.contains("current_census: MaterialPipelineRequirementCensus"));
        assert!(selection.contains("retain_material_pipeline_requirement_misses"));
        assert!(!selection.contains("collect_published_context_pipeline_requirements"));
        assert!(!selection.contains("context_admission_material_count"));
    }

    #[test]
    fn context_selection_precedes_every_gpu_and_command_cache_projection() {
        let build = include_str!("build.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("mesh build test boundary");
        let selection = build
            .find("select_material_generations_for_context(")
            .expect("context material selection");
        let virtual_geometry = build
            .find("build_virtual_geometry_indirect_draw_plan(")
            .expect("virtual geometry plan");
        let gpu_scene = build
            .find("sync_gpu_scene_pending_draws(")
            .expect("GPUScene projection");
        let command_cache = build
            .find("extract_pending_static_mesh_command_cache_hits(")
            .expect("command cache projection");

        assert!(selection < virtual_geometry);
        assert!(selection < gpu_scene);
        assert!(selection < command_cache);
    }

    #[test]
    fn error_proxy_requirements_are_synchronously_admitted_before_selection_returns() {
        let source = include_str!("material_context_admission.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material context admission test boundary");
        let collect = source
            .find("collect_error_proxy_context_pipeline_requirements(")
            .expect("error proxy requirement census");
        let admit = source
            .find("ensure_error_proxy_pipeline_requirements(")
            .expect("synchronous error proxy admission");
        let finish = source
            .rfind("Ok((selection, stats))")
            .expect("fallible context selection result");

        assert!(collect < admit);
        assert!(admit < finish);
    }
}
