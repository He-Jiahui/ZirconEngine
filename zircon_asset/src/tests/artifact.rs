use crate::LibraryCacheKey;

#[test]
fn library_cache_key_changes_when_inputs_change() {
    let baseline = LibraryCacheKey::new("source-a", 1, "config-a");
    let same = LibraryCacheKey::new("source-a", 1, "config-a");
    let changed_source = LibraryCacheKey::new("source-b", 1, "config-a");
    let changed_importer = LibraryCacheKey::new("source-a", 2, "config-a");
    let changed_config = LibraryCacheKey::new("source-a", 1, "config-b");

    assert_eq!(baseline.fingerprint(), same.fingerprint());
    assert_ne!(baseline.fingerprint(), changed_source.fingerprint());
    assert_ne!(baseline.fingerprint(), changed_importer.fingerprint());
    assert_ne!(baseline.fingerprint(), changed_config.fingerprint());
}
