use crate::core::framework::render::ShaderQualityTier;
use crate::graphics::scene::resources::{MaterialDrawGenerationSelection, ResourceStreamer};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::{
    MaterialPipelinePublicationAdmission, MeshPipelineCache,
};
use crate::graphics::types::GraphicsError;

use super::material_draw_selection::MaterialDrawSelection;
use super::material_pipeline_requirements::{
    collect_error_proxy_context_pipeline_requirements,
    collect_previous_context_pipeline_requirements, MaterialPipelineFeatureSet,
    MaterialPipelineRequirementCensus,
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

/// 为本视图选择当前、上一代或错误代理的完整材质；上一代准入不足会转错误代理，错误代理也无法就绪则返回 Err。
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
#[path = "tests/material_context_admission.rs"]
mod tests;
