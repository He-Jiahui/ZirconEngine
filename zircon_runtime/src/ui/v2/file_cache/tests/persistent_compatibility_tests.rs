use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use zircon_runtime_interface::serialization::PositionalEncoding;

#[test]
fn v5_records_rebuild_and_v6_records_preserve_dynamic_values_across_instances() {
    let root = std::env::temp_dir().join(format!(
        "zircon-ui-v2-cache-compatibility-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let source = root.join("assets/ui/compatibility.zui");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(
        &source,
        r#"
[asset]
kind = "view"
id = "ui.cache.compatibility"
version = 2
[root]
node = "root"
[nodes.root]
component = "Label"
props = { text = "Compatible" }
state = { enabled = true, count = 2, scale = 2.5, names = ["one", "two"], created = 2026-10-09T01:02:03Z }
"#,
    )
    .unwrap();
    let cache_root = root.join("cache");
    let mut first = UiV2PrototypeStoreFileCache::with_persistent_cache(cache_root.clone());
    let expected = first.load_store([&source]).unwrap();
    let source_key = UiV2FileStoreCacheKey::from_paths(std::slice::from_ref(&source));
    let current_key = persistent_key_for_source_key("ui-v2-file-cache", &source_key).unwrap();
    let store = UiCompiledArtifactStore::new(cache_root.clone());
    let current_bytes = store.load_payload_bytes(&current_key).unwrap().unwrap();
    let mut legacy: UiV2PersistentFileCacheRecord = serde_json::from_slice(&current_bytes).unwrap();
    assert_eq!(legacy.root_document, *expected.root_document);
    assert_eq!(legacy.compiled, *expected.compiled);

    legacy.record_version = 5;
    let legacy_key = UiCompiledArtifactKey::new(
        current_key.asset_id.clone(),
        current_key.fingerprint,
        5,
        current_key.compiler_version,
    );
    store
        .store_payload_bytes(
            &legacy_key,
            &PositionalEncoding::FixedLittleEndian
                .encode(&legacy)
                .unwrap(),
        )
        .unwrap();
    let current_path = store
        .store_payload_bytes(&current_key, &current_bytes)
        .unwrap();
    std::fs::remove_file(current_path).unwrap();

    let mut rebuilt = UiV2PrototypeStoreFileCache::with_persistent_cache(cache_root.clone());
    let rebuilt = rebuilt.load_store([&source]).unwrap();
    assert!(!rebuilt.persistent_cache_hit);
    assert_eq!(*rebuilt.root_document, *expected.root_document);
    let mut reopened = UiV2PrototypeStoreFileCache::with_persistent_cache(cache_root);
    let reopened = reopened.load_store([&source]).unwrap();
    assert!(reopened.persistent_cache_hit);
    assert_eq!(*reopened.root_document, *expected.root_document);
    assert_eq!(*reopened.compiled, *expected.compiled);
    std::fs::remove_dir_all(root).unwrap();
}
