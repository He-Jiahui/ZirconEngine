#[test]
fn deferred_scene_resources_constructs_the_lighting_pipeline_cache() {
    let source = include_str!("../construct.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("deferred scene resource implementation");

    assert!(implementation.contains("DeferredLightingPipelineCache::new("));
    assert!(!implementation.contains("create_lighting_pipelines("));
}

#[test]
fn deferred_scene_resources_reports_pipeline_and_fallback_startup_separately() {
    let implementation = include_str!("../construct.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("deferred scene resource implementation");

    assert!(implementation.contains("DeferredSceneResourcesStartupReport"));
    assert!(implementation.contains("lighting_pipelines_started"));
    assert!(implementation.contains("fallback_resources_started"));
    assert!(implementation.contains("lighting_pipelines: lighting_pipelines_elapsed"));
    assert!(implementation.contains("fallback_resources: fallback_resources_elapsed"));
}

#[test]
fn deferred_scene_resources_preserves_deferred_lighting_startup_breakdown() {
    let implementation = include_str!("../construct.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("deferred scene resource implementation");

    for expected in [
        "lighting_shader_source_assembly",
        "lighting_pipeline_foundation",
        "lighting_standard_pipeline",
        "lighting_pipeline_startup.shader_source_assembly()",
        "lighting_pipeline_startup.pipeline_foundation()",
        "lighting_pipeline_startup.standard_pipeline()",
    ] {
        assert!(
            implementation.contains(expected),
            "deferred scene startup report must retain `{expected}`"
        );
    }
}
