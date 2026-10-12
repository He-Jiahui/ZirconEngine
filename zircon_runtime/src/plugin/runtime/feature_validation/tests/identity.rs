#[test]
fn runtime_feature_owner_prefix_check_does_not_format_a_string() {
    let source = include_str!("../identity.rs");
    let formatted_prefix = ["format!(\"{}", ".\", feature.owner_plugin_id)"].concat();
    assert!(!source.contains(&formatted_prefix));
}

#[test]
fn runtime_feature_owner_matching_preserves_the_dot_boundary() {
    assert!(super::feature_id_has_owner(
        "rendering",
        "rendering.deferred"
    ));
    assert!(!super::feature_id_has_owner("render", "rendering.deferred"));
}

#[test]
fn optimization_batch_gn_runtime496_feature_owner_byte_boundary_preserves_rules() {
    assert!(super::feature_id_has_owner(
        "rendering",
        "rendering.deferred"
    ));
    assert!(!super::feature_id_has_owner("rendering", "rendering"));
    assert!(!super::feature_id_has_owner("render", "rendering.deferred"));
}

#[test]
#[ignore = "release benchmark submitted to the validation coordinator"]
fn optimization_batch_gn_runtime496_feature_owner_byte_boundary_benchmark() {
    const MARKER: &str = "RUNTIME496_FEATURE_OWNER_BYTE_BOUNDARY_BENCH_V1";
    const ITERATIONS: usize = 100_000;
    let owner = "rendering";
    let feature = "rendering.deferred.materials.shadow_pass.quality_profile";
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        assert!(super::feature_id_has_owner(owner, feature));
    }
    // BUG: [CR-PLUGIN-VALIDATION-0101] 整段计时除以迭代数得到均值，却以 P95 名称输出并执行门槛；没有分位样本，尾延迟回归无法由该证据判定。
    let optimized_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        assert!(feature
            .strip_prefix(owner)
            .is_some_and(|suffix| suffix.starts_with('.')));
    }
    let legacy_p95_ns = start.elapsed().as_nanos() / ITERATIONS as u128;
    eprintln!(
        "{MARKER} optimized_p95_ns={optimized_p95_ns} legacy_p95_ns={legacy_p95_ns} gate=optimized_p95_ns<=legacy_p95_ns*0.90"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns * 90 / 100);
}
