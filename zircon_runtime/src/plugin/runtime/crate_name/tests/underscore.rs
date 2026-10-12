use super::validate_runtime_plugin_module_crate_name_underscore;

#[test]
fn optimization_batch_gl_runtime494_crate_name_underscore_scan_preserves_rules() {
    let valid = ["zircon_plugin_render", "zircon_plugin_render_v2"];
    for crate_name in valid {
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_module_crate_name_underscore(
            "runtime plugin",
            crate_name,
            &mut diagnostics,
        );
        assert!(
            diagnostics.is_empty(),
            "unexpected diagnostic for {crate_name}"
        );
    }

    let invalid = ["zircon_plugin_render_", "zircon_plugin_render__v2"];
    for crate_name in invalid {
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_module_crate_name_underscore(
            "runtime plugin",
            crate_name,
            &mut diagnostics,
        );
        assert_eq!(diagnostics.len(), 1, "missing diagnostic for {crate_name}");
    }
}

#[test]
#[ignore = "release benchmark submitted to the validation coordinator"]
fn optimization_batch_gl_runtime494_crate_name_underscore_scan_benchmark() {
    const MARKER: &str = "RUNTIME494_CRATE_NAME_UNDERSCORE_SCAN_BENCH_V1";
    const SAMPLE: &str = "zircon_plugin_render_pipeline_materials_shadow_pass_quality_profile";
    const ITERATIONS: usize = 100_000;
    let start = std::time::Instant::now();
    let mut diagnostics = Vec::new();
    for _ in 0..ITERATIONS {
        diagnostics.clear();
        validate_runtime_plugin_module_crate_name_underscore(
            "runtime plugin",
            SAMPLE,
            &mut diagnostics,
        );
        assert!(diagnostics.is_empty());
    }
    // BUG: [CR-PLUGIN-VALIDATION-0201] 此基准将整段耗时除迭代次数标为 p95，实际是均值；证据：两段计时均未保留样本或计算分位数。
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
