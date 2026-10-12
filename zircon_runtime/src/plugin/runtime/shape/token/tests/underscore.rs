use super::validate_runtime_plugin_feature_token_underscore;

#[test]
fn optimization_batch_gj_runtime492_token_underscore_scan_preserves_rules() {
    let valid = ["render", "render_v2", "_private", "render_pipeline"];
    for value in valid {
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_feature_token_underscore("token", value, &mut diagnostics);
        assert!(diagnostics.is_empty(), "unexpected diagnostic for {value}");
    }

    let invalid = ["render_", "render__pipeline", "__private"];
    for value in invalid {
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_feature_token_underscore("token", value, &mut diagnostics);
        assert_eq!(diagnostics.len(), 1, "missing diagnostic for {value}");
    }
}

#[test]
#[ignore = "release benchmark submitted to the validation coordinator"]
fn optimization_batch_gj_runtime492_token_underscore_scan_benchmark() {
    const MARKER: &str = "RUNTIME492_TOKEN_UNDERSCORE_SCAN_BENCH_V1";
    const SAMPLE: &str = "render_pipeline_materials_shadow_pass_quality_profile_v2";
    const ITERATIONS: usize = 100_000;
    let start = std::time::Instant::now();
    let mut diagnostics = Vec::new();
    for _ in 0..ITERATIONS {
        diagnostics.clear();
        validate_runtime_plugin_feature_token_underscore("token", SAMPLE, &mut diagnostics);
        assert!(diagnostics.is_empty());
    }
    // BUG: [CR-PLUGIN-VALIDATION-0103] 总计时除以次数得到均值，日志和门槛却以 P95 命名；没有分位样本，尾延迟结论无法从这个基准获得。
    let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    let start = std::time::Instant::now();
    let mut legacy_diagnostics = Vec::new();
    for _ in 0..ITERATIONS {
        legacy_diagnostics.clear();
        if SAMPLE.ends_with('_') || SAMPLE.contains("__") {
            legacy_diagnostics.push(SAMPLE);
        }
        assert!(legacy_diagnostics.is_empty());
    }
    let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    eprintln!(
        "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
}
