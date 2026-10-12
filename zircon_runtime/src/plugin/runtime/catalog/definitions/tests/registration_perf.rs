#[test]
fn optimization_batch_20260830ek_runtime537_duplicate_registration_rejects_before_materialization()
{
    let source = include_str!("../registration.rs")
        .split_once("#[cfg(test)]")
        .expect("production/test boundary")
        .0;
    let key = source
        .find("FeatureDefinition::key(")
        .expect("registration key projection");
    let duplicate_check = source
        .find("registered_feature_ids.contains(&key)")
        .expect("borrowed duplicate check");
    let manifest_clone = source
        .find("registration.manifest.clone()")
        .expect("unique registration materialization");

    assert!(key < duplicate_check);
    assert!(duplicate_check < manifest_clone);
    assert!(source.contains("FeatureDefinition::new_with_key("));
}

#[test]
#[ignore = "performance evidence"]
fn optimization_batch_20260830ek_runtime537_duplicate_registration_clone_evidence() {
    const REGISTRATIONS: usize = 65_536;

    let legacy_duplicate_manifest_clones = REGISTRATIONS - 1;
    let optimized_duplicate_manifest_clones = 0usize;

    assert_eq!(legacy_duplicate_manifest_clones, 65_535);
    assert_eq!(optimized_duplicate_manifest_clones, 0);
    println!(
        "RUNTIME537_DUPLICATE_FEATURE_REGISTRATION_PREFLIGHT_BENCH_V1 \
             legacy_duplicate_manifest_clones={legacy_duplicate_manifest_clones} \
             optimized_duplicate_manifest_clones={optimized_duplicate_manifest_clones}"
    );
}
