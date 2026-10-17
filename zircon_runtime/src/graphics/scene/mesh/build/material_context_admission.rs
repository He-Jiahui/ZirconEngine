use crate::core::framework::render::ShaderQualityTier;
use crate::graphics::scene::resources::{MaterialDrawGenerationSelection, ResourceStreamer};
use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::{
    MaterialPipelinePublicationAdmission, MaterialPipelineRequirementSet, MeshPipelineCache,
};
use crate::graphics::types::GraphicsError;

use super::material_draw_selection::MaterialDrawSelection;
use super::material_pipeline_requirements::{
    collect_error_proxy_context_pipeline_requirements, collect_material_pipeline_requirements,
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
    pub(super) cold_staged_candidate_count: usize,
    pub(super) cold_staged_selected_count: usize,
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

/// 为本视图选择完整材质代际；冷启动候选须通过本视图管线准入，热替换保留 Published。
/// 旧代际准入不足会转错误代理，错误代理也无法就绪则返回 Err。
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

    let mut current_rows = current_census.into_requirements().collect::<Vec<_>>();
    current_rows.sort_unstable_by_key(|(material_id, generation, _)| (*material_id, *generation));
    let current_admissions =
        admit_frozen_rows(device, streamer, mesh_pipelines, &current_rows, &mut stats);
    // Pending draws from a cold material contain an error proxy and no Published generation.
    // Reuse the real staged census to recover the candidate key and every observed geometry context.
    let mut cold_rows = collect_material_pipeline_requirements(
        pending_draws,
        streamer,
        features,
        shader_quality,
        volumetric_fog_enabled,
    )
    .into_requirements()
    .filter(|(id, generation, requirements)| {
        requirements.len() != 0
            && streamer
                .admitted_cold_staged_material_draw_proxy(id, *generation)
                .draw_generation()
                == Some(*generation)
    })
    .collect::<Vec<_>>();
    cold_rows.sort_unstable_by_key(|(id, generation, _)| (*id, *generation));
    stats.cold_staged_candidate_count = cold_rows.len();
    stats.tracked_material_count = stats.tracked_material_count.saturating_add(cold_rows.len());
    let cold_admissions =
        admit_frozen_rows(device, streamer, mesh_pipelines, &cold_rows, &mut stats);
    for (material_id, _) in &cold_admissions {
        // Keep mandatory fallback admission intact even if the first candidate attempt is Ready.
        selection.select(*material_id, MaterialDrawGenerationSelection::ErrorProxy);
        stats.error_proxy_selected_count = stats.error_proxy_selected_count.saturating_add(1);
    }
    for &(material_id, admission) in &current_admissions {
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

    retry_current_requirements_after_error_proxy_validation(
        device,
        streamer,
        mesh_pipelines,
        &current_rows,
        &current_admissions,
        &mut selection,
        &mut stats,
    );

    select_cold_staged_requirements_after_error_proxy_validation(
        device,
        streamer,
        mesh_pipelines,
        &cold_rows,
        &cold_admissions,
        &mut selection,
        &mut stats,
    );

    crate::profile_counter!(
        "render",
        "material_context_admission_cold_staged_candidate_count",
        stats.cold_staged_candidate_count
    );
    crate::profile_counter!(
        "render",
        "material_context_admission_cold_staged_selected_count",
        stats.cold_staged_selected_count
    );

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
    admit_frozen_rows(device, streamer, mesh_pipelines, &rows, stats)
}

fn admit_frozen_rows(
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    mesh_pipelines: &mut MeshPipelineCache,
    rows: &[FrozenMaterialRequirementRow],
    stats: &mut MaterialContextAdmissionStats,
) -> Vec<(
    crate::core::resource::ResourceId,
    MaterialPipelinePublicationAdmission,
)> {
    rows.iter()
        .map(|(material_id, generation, requirements)| {
            let admission = mesh_pipelines.ensure_material_pipeline_requirements_for_generation(
                device,
                streamer,
                *material_id,
                *generation,
                requirements,
            );
            record_material_pipeline_admission(stats, admission);
            (*material_id, admission)
        })
        .collect()
}

// These rows retain the exact generation and requirement set observed by extraction.
type FrozenMaterialRequirementRow = (
    crate::core::resource::ResourceId,
    u64,
    MaterialPipelineRequirementSet,
);

fn record_material_pipeline_admission(
    stats: &mut MaterialContextAdmissionStats,
    admission: MaterialPipelinePublicationAdmission,
) {
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
}

fn current_admission_can_retry(admission: MaterialPipelinePublicationAdmission) -> bool {
    use crate::graphics::pipeline::PipelineAdmissionReason;
    matches!(
        admission,
        MaterialPipelinePublicationAdmission::Deferred { unavailable, .. }
            if matches!(unavailable.reason(),
                PipelineAdmissionReason::SourceValidationQueued
                    | PipelineAdmissionReason::SourceValidationPending
                    | PipelineAdmissionReason::QueueSaturated)
    )
}

fn select_cold_staged_requirements_after_error_proxy_validation(
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    mesh_pipelines: &mut MeshPipelineCache,
    rows: &[FrozenMaterialRequirementRow],
    initial: &[(
        crate::core::resource::ResourceId,
        MaterialPipelinePublicationAdmission,
    )],
    selection: &mut MaterialDrawSelection,
    stats: &mut MaterialContextAdmissionStats,
) {
    debug_assert_eq!(rows.len(), initial.len());
    for ((material_id, generation, requirements), (initial_id, first)) in rows.iter().zip(initial) {
        debug_assert_eq!(material_id, initial_id);
        // Observe the exact frozen row once more only when the existing mandatory fallback
        // validation could have completed its source work. This adds no wait or global publication.
        let final_admission = if current_admission_can_retry(*first) {
            let admission = mesh_pipelines.ensure_material_pipeline_requirements_for_generation(
                device,
                streamer,
                *material_id,
                *generation,
                requirements,
            );
            record_material_pipeline_admission(stats, admission);
            admission
        } else {
            *first
        };
        if matches!(
            final_admission,
            MaterialPipelinePublicationAdmission::Ready { .. }
        ) && streamer
            .admitted_cold_staged_material_draw_proxy(material_id, *generation)
            .draw_generation()
            == Some(*generation)
        {
            if selection.selection_for(material_id) == MaterialDrawGenerationSelection::ErrorProxy {
                stats.error_proxy_selected_count =
                    stats.error_proxy_selected_count.saturating_sub(1);
            }
            selection.select_admitted_cold_staged(*material_id, *generation);
            stats.cold_staged_selected_count = stats.cold_staged_selected_count.saturating_add(1);
        }
        #[cfg(test)]
        observe_cold_material_selection_for_test(
            streamer,
            selection,
            *material_id,
            *generation,
            *first,
            final_admission,
        );
    }
}

fn retry_current_requirements_after_error_proxy_validation(
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    mesh_pipelines: &mut MeshPipelineCache,
    rows: &[FrozenMaterialRequirementRow],
    initial: &[(
        crate::core::resource::ResourceId,
        MaterialPipelinePublicationAdmission,
    )],
    selection: &mut MaterialDrawSelection,
    stats: &mut MaterialContextAdmissionStats,
) {
    debug_assert_eq!(rows.len(), initial.len());
    for ((material_id, generation, requirements), (initial_id, first)) in rows.iter().zip(initial) {
        debug_assert_eq!(material_id, initial_id);
        // This is a nonblocking observation. The mandatory error-proxy admission above
        // may already have completed source validation for this same frozen context.
        let final_admission = if current_admission_can_retry(*first) {
            let admission = mesh_pipelines.ensure_material_pipeline_requirements_for_generation(
                device,
                streamer,
                *material_id,
                *generation,
                requirements,
            );
            record_material_pipeline_admission(stats, admission);
            admission
        } else {
            *first
        };
        if !matches!(first, MaterialPipelinePublicationAdmission::Ready { .. })
            && matches!(
                final_admission,
                MaterialPipelinePublicationAdmission::Ready { .. }
            )
        {
            match selection.selection_for(material_id) {
                MaterialDrawGenerationSelection::PreviousPublished => {
                    stats.previous_selected_count = stats.previous_selected_count.saturating_sub(1);
                }
                MaterialDrawGenerationSelection::ErrorProxy => {
                    stats.error_proxy_selected_count =
                        stats.error_proxy_selected_count.saturating_sub(1);
                }
                MaterialDrawGenerationSelection::Published => {}
            }
            selection.select(*material_id, MaterialDrawGenerationSelection::Published);
            stats.current_ready_count = stats.current_ready_count.saturating_add(1);
        }
        #[cfg(test)]
        observe_material_selection_for_test(
            streamer,
            *material_id,
            *generation,
            *first,
            final_admission,
            selection.selection_for(material_id),
        );
    }
}

#[cfg(test)]
fn observe_material_selection_for_test(
    streamer: &ResourceStreamer,
    material_id: crate::core::resource::ResourceId,
    frozen_generation: u64,
    first: MaterialPipelinePublicationAdmission,
    final_admission: MaterialPipelinePublicationAdmission,
    selected: MaterialDrawGenerationSelection,
) {
    if std::env::var_os("ZIRCON_RENDER_MATERIAL_SELECTION_DIAGNOSTICS").as_deref()
        != Some(std::ffi::OsStr::new("1"))
    {
        return;
    }
    // These are selection-stage values, not evidence of a submitted draw.
    let proxy = streamer.material_draw_proxy(&material_id, selected);
    let selected_generation = proxy.draw_generation();
    let runtime = proxy.runtime();
    eprintln!(
        "material-selection-stage material={material_id:?} frozen_generation={frozen_generation} first={first:?} final={final_admission:?} selected={selected:?} selected_proxy_generation={selected_generation:?} selected_proxy_tint_shading_profile={:?} submitted_draw_observed=false",
        runtime.map(|value| (value.base_color, value.shading_model_id, value.subsurface_profile_index)),
    );
}

#[cfg(test)]
fn observe_cold_material_selection_for_test(
    streamer: &ResourceStreamer,
    selection: &MaterialDrawSelection,
    material_id: crate::core::resource::ResourceId,
    frozen_generation: u64,
    first: MaterialPipelinePublicationAdmission,
    final_admission: MaterialPipelinePublicationAdmission,
) {
    if std::env::var_os("ZIRCON_RENDER_MATERIAL_SELECTION_DIAGNOSTICS").as_deref()
        != Some(std::ffi::OsStr::new("1"))
    {
        return;
    }
    let proxy = selection.proxy(streamer, &material_id);
    eprintln!(
        "cold-material-selection-stage material={material_id:?} frozen_generation={frozen_generation} first={first:?} final={final_admission:?} selected_proxy_generation={:?} selected_proxy_tint_shading_profile={:?} globally_published_generation={:?} submitted_draw_observed=false",
        proxy.draw_generation(),
        proxy.runtime().map(|value| (value.base_color, value.shading_model_id, value.subsurface_profile_index)),
        streamer.published_material_draw_proxy(&material_id).draw_generation(),
    );
}

#[cfg(test)]
#[path = "tests/material_context_admission.rs"]
mod tests;
