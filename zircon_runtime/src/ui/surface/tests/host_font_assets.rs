use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

struct HostFontTestDirectory(PathBuf);

impl HostFontTestDirectory {
    fn new() -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("host test clock must follow the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "zircon-host-font-test-{}-{suffix}",
            std::process::id()
        ));
        std::fs::create_dir(&path).expect("host font test directory must be unique");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for HostFontTestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0.join("font.ttc"));
        let _ = std::fs::remove_file(self.0.join("host.font.toml"));
        let _ = std::fs::remove_dir(&self.0);
    }
}

#[test]
fn host_font_registry_deduplicates_borrowed_refs_before_arc_admission() {
    let source = include_str!("../host_font_assets.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("host font production source");

    assert!(
        production.contains("let mut asset_refs = asset_refs.to_vec();"),
        "host font admission should retain borrowed input refs until a cache miss"
    );
    assert!(production.contains("asset_refs.sort_unstable();"));
    assert!(production.contains("asset_refs.dedup();"));
    assert!(
        !production.contains("collect::<BTreeSet<_>>()"),
        "duplicate admission must not allocate a BTreeSet of owned Arcs"
    );
}

#[test]
fn host_font_admission_failure_does_not_claim_readiness() {
    let collection = FontCollectionService::new();
    let original = collection.collection_snapshot().database().face_count();
    let error = HostFontAssetRegistry::default()
        .load(
            Arc::clone(&collection),
            &["res://fonts/missing-host-review-font.font.toml"],
        )
        .expect_err("missing font must fail admission");
    assert!(error.asset_ref.contains("missing-host-review-font"));
    assert_eq!(
        collection.collection_snapshot().database().face_count(),
        original
    );
}

#[test]
fn live_host_font_is_shared_without_reopening_and_released_with_its_last_owner() {
    let directory = HostFontTestDirectory::new();
    let source = directory.path().join("font.ttc");
    std::fs::write(
        &source,
        include_bytes!("../../../../assets/fonts/ZirconDefaultComposite-subset.ttc"),
    )
    .unwrap();
    let manifest = directory.path().join("host.font.toml");
    std::fs::write(
        &manifest,
        "source = 'font.ttc'\nfamily = 'Host font lifecycle test'\n",
    )
    .unwrap();
    let reference = manifest.to_str().unwrap();
    let collection = FontCollectionService::new();
    let registry = HostFontAssetRegistry::default();
    let first = registry
        .load(Arc::clone(&collection), &[reference, reference])
        .unwrap();
    assert_eq!(first.ready().len(), 1);
    let revision = collection.generation();
    std::fs::remove_file(&manifest).unwrap();
    let second = registry
        .load(Arc::clone(&collection), &[reference])
        .unwrap();
    assert!(Arc::ptr_eq(&first._residents[0], &second._residents[0]));
    assert_eq!(collection.generation(), revision);
    drop(first);
    assert!(collection
        .collection_snapshot()
        .database()
        .font_asset_primary_face(reference)
        .is_some());
    drop(second);
    assert!(collection
        .collection_snapshot()
        .database()
        .font_asset_primary_face(reference)
        .is_none());
    assert!(registry.load(collection, &[reference]).is_err());
}
