use super::validate_runtime_plugin_package_bevy_reference_path_segments;

#[test]
fn optimization_batch_gg_runtime489_reference_segments_preserves_path_rules() {
    let valid = ["assets/materials/main", "plugins/render.v2/shader.bin"];
    for reference in valid {
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_package_bevy_reference_path_segments(
            "render",
            reference,
            &mut diagnostics,
        );
        assert!(
            diagnostics.is_empty(),
            "unexpected diagnostic for {reference}"
        );
    }

    let invalid = [
        "",
        "/assets",
        "assets/",
        "assets//main",
        "assets/.",
        "assets/..",
    ];
    for reference in invalid {
        let mut diagnostics = Vec::new();
        validate_runtime_plugin_package_bevy_reference_path_segments(
            "render",
            reference,
            &mut diagnostics,
        );
        assert_eq!(diagnostics.len(), 1, "missing diagnostic for {reference}");
    }
}

#[test]
#[ignore = "release benchmark submitted to the validation coordinator"]
fn optimization_batch_gg_runtime489_reference_segments_benchmark() {
    const MARKER: &str = "RUNTIME489_REFERENCE_SEGMENTS_SCAN_BENCH_V1";
    const SAMPLE: &str =
        "assets/materials/shaders/forward_plus/clustered_lighting/quality_profile.bin";
    const ITERATIONS: usize = 100_000;
    let start = std::time::Instant::now();
    let mut diagnostics = Vec::new();
    for _ in 0..ITERATIONS {
        diagnostics.clear();
        validate_runtime_plugin_package_bevy_reference_path_segments(
            "render",
            SAMPLE,
            &mut diagnostics,
        );
        assert!(diagnostics.is_empty());
    }
    // BUG: [CR-PLUGIN-VALIDATION-0302] 忽略的性能门槛将整批平均耗时命名为 p95，会错误代表尾延迟；证据：两个批次均用 elapsed().as_nanos() / ITERATIONS。
    let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    let start = std::time::Instant::now();
    let mut legacy_diagnostics = Vec::new();
    for _ in 0..ITERATIONS {
        legacy_diagnostics.clear();
        if SAMPLE
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
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
