#[test]
fn direct_mesh_preparation_combines_profile_and_preview_light_policies() {
    let source = include_str!("../build_mesh_draws.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("mesh preparation source should retain a test-module boundary");
    let direct_method = production
        .split("fn build_mesh_draws(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn build_mesh_draws_with_command_cache(")
                .next()
        })
        .expect("direct mesh preparation method");

    assert!(direct_method.contains("uses_direct_lights: bool,"));
    assert!(
        direct_method.contains("uses_direct_lights.then_some(frame.preview().lighting_enabled)")
    );
    assert!(direct_method
        .contains("material_pipeline_features,\n            direct_lighting_preparation,"));
}

#[test]
fn compiled_mesh_preparation_retains_its_preview_light_policy() {
    let source = include_str!("../build_mesh_draws.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("mesh preparation source should retain a test-module boundary");
    let compiled_method = production
        .split("fn build_mesh_draws_with_command_cache(")
        .nth(1)
        .expect("compiled mesh preparation method");

    assert!(compiled_method.contains(
        "material_pipeline_features,\n            Some(frame.preview().lighting_enabled),"
    ));
}

#[test]
fn hit_proxy_mesh_preparation_is_an_isolated_on_demand_profile() {
    let source = include_str!("../build_mesh_draws.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("mesh preparation source should retain a test-module boundary");
    let hit_proxy_method = production
        .split("fn build_hit_proxy_mesh_draws(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn build_mesh_draws_with_command_cache(")
                .next()
        })
        .expect("hit-proxy mesh preparation method");

    assert!(hit_proxy_method.contains("MaterialPipelineFeatureSet::hit_proxy(policy)"));
    assert!(hit_proxy_method.contains("Some(hit_proxy_tokens)"));
    assert!(hit_proxy_method.contains("None,\n            None,\n            None,"));
}

#[test]
fn environment_capture_builds_one_reflected_scene_draw_set_without_snapshot_sidebands() {
    let source = include_str!("../build_mesh_draws.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("mesh preparation source should retain a test-module boundary");
    let capture_method = production
        .split("fn build_environment_capture_mesh_draws(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn build_mesh_draws_with_command_cache(")
                .next()
        })
        .expect("environment capture mesh preparation method");

    assert!(capture_method.contains("MaterialPipelineFeatureSet::environment_capture()"));
    assert!(!capture_method.contains("Some(frame.preview().lighting_enabled)"));
    assert!(capture_method.contains("frame,\n            false,\n            false,"));
    assert!(capture_method.contains(
        "MaterialPipelineFeatureSet::environment_capture(),\n            None,\n            None,\n            None,\n            None,"
    ));
}
