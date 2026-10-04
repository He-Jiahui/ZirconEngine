use crate::graphics::scene::resources::MaterialDrawGenerationSelection;

use super::{generation_selection_for_current, MaterialGenerationRequirementCacheStats};

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
    let source = include_str!("../material_context_admission.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material context admission test boundary");

    assert!(source.contains("MaterialPipelinePublicationAdmission::Deferred"));
    assert!(source.contains("MaterialPipelinePublicationAdmission::Failed"));
    assert!(source.contains("MaterialDrawGenerationSelection::PreviousPublished"));
    assert!(source.contains("MaterialDrawGenerationSelection::ErrorProxy"));
    assert!(source.contains("collect_previous_context_pipeline_requirements"));
    assert!(source.contains("ensure_material_pipeline_requirements_for_generation"));
    // BUG: [CR-R02-runtime_wave12_graphics_mesh_draw_build-0002] 当前生产准入读取已冻结的 census 代际，已无 draw_generation()；前六个断言均成立后本行固定失败，守卫须追踪实际收集入口。
    assert!(source.contains("draw_generation()"));
    assert!(source.contains("\"material\", \"context_admission\""));
}

#[test]
fn context_admission_profiles_fused_census_and_generation_cache_scale() {
    let source = include_str!("../material_context_admission.rs")
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
    assert!(source.contains("material_context_admission_generation_cache_hit_requirement_count"));
    assert!(source.contains("material_context_admission_generation_cache_miss_requirement_count"));
}

#[test]
fn context_admission_consumes_fused_generation_ledger_misses() {
    let source = include_str!("../material_context_admission.rs")
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
    let build = include_str!("../build.rs")
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
    let source = include_str!("../material_context_admission.rs")
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
