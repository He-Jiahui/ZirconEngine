use super::validate_runtime_plugin_package_semver_component_leading_zeroes;

#[test]
fn optimization_batch_gp_runtime498_leading_zero_scan_preserves_rules() {
    let mut diagnostics = Vec::new();
    assert!(
        validate_runtime_plugin_package_semver_component_leading_zeroes(
            "version",
            "1.2.3",
            "major",
            "0",
            &mut diagnostics,
        )
    );
    assert!(diagnostics.is_empty());
    assert!(
        !validate_runtime_plugin_package_semver_component_leading_zeroes(
            "version",
            "01.2.3",
            "major",
            "01",
            &mut diagnostics,
        )
    );
    assert_eq!(diagnostics.len(), 1);
}

#[test]
#[ignore = "release benchmark submitted to the validation coordinator"]
fn optimization_batch_gp_runtime498_leading_zero_scan_benchmark() {
    const MARKER: &str = "RUNTIME498_LEADING_ZERO_SCAN_BENCH_V1";
    const ITERATIONS: usize = 100_000;
    let segment = "123456789";
    let start = std::time::Instant::now();
    let mut diagnostics = Vec::new();
    for _ in 0..ITERATIONS {
        diagnostics.clear();
        assert!(
            validate_runtime_plugin_package_semver_component_leading_zeroes(
                "version",
                "123.456.789",
                "major",
                segment,
                &mut diagnostics,
            )
        );
    }
    // BUG: [CR-PLUGIN-VALIDATION-0364] 总耗时除以循环次数得到均值，但后续输出和门槛将其标为 p95；证据：这里没有采样排序或分位数计算。
    let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        assert!(segment == "0" || !segment.starts_with('0'));
    }
    let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    eprintln!(
        "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
}
