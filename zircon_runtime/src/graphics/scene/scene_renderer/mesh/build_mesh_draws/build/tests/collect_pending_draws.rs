#[test]
fn published_requirement_census_observes_each_new_draw_inside_collection() {
    let source = include_str!("../collect_pending_draws.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("pending draw collection test boundary");

    assert!(source.contains("collect_pending_draws_with_published_pipeline_requirements"));
    assert!(source.contains("first_pending_draw"));
    assert!(source.contains("&pending_draws[first_pending_draw..]"));
    assert!(source.contains("PublishedMaterialPipelineRequirementCollector"));
    assert!(source.contains("collector.observe_published_draw"));
    assert!(!source.contains("insert_published_material_pipeline_requirements"));
}
