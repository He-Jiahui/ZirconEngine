#[test]
fn deferred_lighting_pipeline_targets_use_fixed_stack_storage() {
    let source = include_str!("../create.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("deferred lighting pipeline implementation");

    assert!(!implementation.contains("let mut targets = vec!["));
    assert!(implementation.contains("let mut targets = ["));
}

#[test]
fn deferred_lighting_pipeline_compiles_the_fullscreen_vertex_stage_separately() {
    let source = include_str!("../create.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("deferred lighting pipeline implementation");

    assert!(implementation.contains("DEFERRED_LIGHTING_FULLSCREEN_VERTEX_SHADER"));
    assert!(implementation.contains("fn fs_main(@builtin(position) _position"));
    assert!(implementation.contains("fn fs_main_sss(@builtin(position) _position"));
    assert!(implementation.contains("module: vertex_shader,"));
    assert!(implementation.contains("module: fragment_shader,"));
}

#[test]
fn deferred_lighting_pipeline_cache_prewarms_only_full_scene_standard_pipeline() {
    let source = include_str!("../create.rs");
    let cache = source
        .split("impl DeferredLightingPipelineCache")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn create_lighting_pipeline_from_foundation")
                .next()
        })
        .expect("deferred lighting cache implementation");

    // BUG: [CR-W12-RENDER-AUX-A-0004] 此测试只截取 impl 到创建函数之间，遗漏 impl 之前的字段声明；
    // 当前截段中待查字段类型匹配数为零，因此该 assert 必失败，无法检查预热策略。
    assert!(cache.contains("OnceLock<wgpu::RenderPipeline>"));
    assert!(cache.contains("pipeline.get_or_init"));
    assert!(cache.contains("foundation.get_or_init"));
    assert!(cache.contains("cache.pipeline_from_foundation(device, foundation, false)"));
    assert!(!cache.contains("cache.pipeline_from_foundation(device, foundation, true)"));
    assert!(cache.contains(
        "deferred_lighting_profile\n            == SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview"
    ));
    assert!(cache.contains("(Duration::ZERO, Duration::ZERO)"));
}

#[test]
fn deferred_lighting_pipeline_reports_source_foundation_and_standard_pso_separately() {
    let implementation = include_str!("../create.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("deferred lighting pipeline implementation");

    for expected in [
        "DeferredLightingPipelineStartupReport",
        "shader_source_assembly",
        "pipeline_foundation",
        "standard_pipeline",
        "let shader_source_started = Instant::now();",
        "let foundation_started = Instant::now();",
        "let standard_pipeline_started = Instant::now();",
    ] {
        assert!(
            implementation.contains(expected),
            "deferred lighting startup profiling must retain `{expected}`"
        );
    }
}
