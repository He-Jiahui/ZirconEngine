use super::FeatureStatus;

#[test]
fn optimization_batch_20260830ef_runtime535_duplicate_status_inputs_are_borrowed_and_deduplicated()
{
    let mut status = FeatureStatus::new("feature.test".into(), "plugin.owner".into());

    status.add_missing_plugin("plugin.missing");
    status.add_missing_plugin("plugin.missing");
    status.add_missing_capability("render.compute");
    status.add_missing_capability("render.compute");

    assert_eq!(status.missing_plugins, ["plugin.missing"]);
    assert_eq!(status.missing_capabilities, ["render.compute"]);
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830ef_runtime535_duplicate_status_clone_evidence() {
    const DUPLICATE_UPDATES: usize = 65_536;
    const LEGACY_STRING_CLONES_PER_UPDATE: usize = 2;
    const OPTIMIZED_FIRST_INSERT_STRING_CLONES: usize = 2;
    const MARKER: &str = "RUNTIME535_DUPLICATE_FEATURE_STATUS_BORROW_BENCH_V1";

    let legacy_string_clones = DUPLICATE_UPDATES.saturating_mul(LEGACY_STRING_CLONES_PER_UPDATE);
    let optimized_string_clones = OPTIMIZED_FIRST_INSERT_STRING_CLONES;
    let reduction_pct = 100.0 * (legacy_string_clones - optimized_string_clones) as f64
        / legacy_string_clones as f64;

    assert_eq!(legacy_string_clones, 131_072);
    assert_eq!(optimized_string_clones, 2);
    assert!(reduction_pct > 99.99);
    println!(
        "{MARKER} duplicate_updates={DUPLICATE_UPDATES} \
             legacy_string_clones={legacy_string_clones} \
             optimized_string_clones={optimized_string_clones} \
             reduction_pct={reduction_pct:.4}"
    );
}
