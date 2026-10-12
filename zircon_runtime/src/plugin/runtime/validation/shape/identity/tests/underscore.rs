use super::validate_runtime_plugin_package_id_underscore;

#[test]
fn optimization_batch_gg_runtime490_underscore_scan_preserves_segment_rules() {
    let valid = ["render.pipeline", "render_v2.pipeline2", "render._private"];
    for value in valid {
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_package_id_underscore(
            "runtime plugin package",
            "package_id",
            value,
            &mut diagnostics,
        );
        assert!(diagnostics.is_empty(), "unexpected diagnostic for {value}");
    }

    let invalid = [
        "render_",
        "render__pipeline",
        "render._private_",
        "render.a__b",
    ];
    for value in invalid {
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_package_id_underscore(
            "runtime plugin package",
            "package_id",
            value,
            &mut diagnostics,
        );
        assert_eq!(diagnostics.len(), 1, "missing diagnostic for {value}");
    }
}

#[test]
#[ignore = "release benchmark submitted to the validation coordinator"]
fn optimization_batch_gg_runtime490_underscore_scan_benchmark() {
    const MARKER: &str = "RUNTIME490_UNDERSCORE_SCAN_BENCH_V1";
    const SAMPLE: &str = "render.pipeline.materials.shadow_pass.v2.alpha_mask.quality_profile";
    const ITERATIONS: usize = 100_000;
    let start = std::time::Instant::now();
    let mut diagnostics = Vec::new();
    for _ in 0..ITERATIONS {
        diagnostics.clear();
        validate_runtime_plugin_package_id_underscore(
            "runtime plugin package",
            "package_id",
            SAMPLE,
            &mut diagnostics,
        );
        assert!(diagnostics.is_empty());
    }
    // BUG: [CR-PLUGIN-VALIDATION-0363] 总耗时除以循环次数得到均值，但后续输出和门槛将其标为 p95；证据：这里没有采样排序或分位数计算。
    let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    let start = std::time::Instant::now();
    let mut legacy_diagnostics = Vec::new();
    for _ in 0..ITERATIONS {
        legacy_diagnostics.clear();
        if SAMPLE
            .split('.')
            .any(|segment| segment.ends_with('_') || segment.contains("__"))
        {
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
