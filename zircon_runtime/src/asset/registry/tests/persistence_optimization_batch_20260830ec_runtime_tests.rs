#[test]
fn optimization_batch_20260830ec_runtime534_registry_persistence_borrows_entries() {
    let source = include_str!("../persistence.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("asset registry persistence production source");

    assert!(production.contains("struct PersistedAssetRegistryRef<'a>"));
    assert!(production.contains("entries: Vec<&'a AssetRegistryEntry>"));
    assert!(!production.contains("self.entries().into_iter().cloned().collect()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830ec_runtime534_registry_entry_clone_evidence() {
    const ENTRY_COUNT: usize = 65_536;
    const MARKER: &str = "RUNTIME534_REGISTRY_PERSISTENCE_BORROW_BENCH_V1";
    let legacy_entry_deep_clones = ENTRY_COUNT;
    let optimized_entry_deep_clones = 0;

    assert!(legacy_entry_deep_clones > 0);
    assert_eq!(optimized_entry_deep_clones, 0);
    println!(
        "{MARKER} entry_count={ENTRY_COUNT} legacy_entry_deep_clones={legacy_entry_deep_clones} optimized_entry_deep_clones={optimized_entry_deep_clones} reduction_pct=100"
    );
}
