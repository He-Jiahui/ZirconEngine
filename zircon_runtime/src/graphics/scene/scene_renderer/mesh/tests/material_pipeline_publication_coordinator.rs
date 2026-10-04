#[test]
fn coordinator_source_keeps_publication_behind_all_ready_admission() {
    let source = include_str!("../material_pipeline_publication_coordinator.rs")
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
