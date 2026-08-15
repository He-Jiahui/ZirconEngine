use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;

use crate::core::recovery::{
    AutosaveDocumentId, AutosaveError, AutosaveExtension, AutosaveSourcePath, AutosaveStore,
};

fn document(value: &str) -> AutosaveDocumentId {
    AutosaveDocumentId::parse(value).expect("test document id should be valid")
}

fn extension(value: &str) -> AutosaveExtension {
    AutosaveExtension::parse(value).expect("test extension should be valid")
}

fn source_path(value: &str) -> AutosaveSourcePath {
    AutosaveSourcePath::parse(value).expect("test recovery source path should be valid")
}

#[test]
fn recovery_source_path_rejects_absolute_and_traversal_paths() {
    assert!(matches!(
        AutosaveSourcePath::parse("../outside.zscene"),
        Err(AutosaveError::InvalidRecoverySourcePath { .. })
    ));
    assert!(matches!(
        AutosaveSourcePath::parse(std::env::temp_dir().join("outside.zscene")),
        Err(AutosaveError::InvalidRecoverySourcePath { .. })
    ));
}

#[test]
fn recovery_catalog_rebuilds_the_latest_snapshot_for_its_source_document() {
    let root = temporary_root("discover-latest");
    let source = source_path("scenes/main.zscene");
    let source_file = root.join(source.as_path());
    fs::create_dir_all(source_file.parent().expect("source path has a parent")).unwrap();
    fs::write(&source_file, b"authoritative source").unwrap();
    let store = AutosaveStore::new(&root);
    let document = document("scene_main");

    store
        .write_snapshot(&document, &source, 1, &extension("zscene"), b"first")
        .unwrap();
    let latest = store
        .write_snapshot(&document, &source, 2, &extension("zscene"), b"second")
        .unwrap();

    let candidates = store.recovery_candidates().unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].document(), &document);
    assert_eq!(candidates[0].source_path(), source_file.as_path());
    assert_eq!(candidates[0].autosave_path(), latest.as_path());
    remove_temporary_root(&root);
}

#[test]
fn recovery_catalog_rejects_silent_document_source_rebinding() {
    let root = temporary_root("source-conflict");
    let store = AutosaveStore::new(&root);
    let document = document("scene_main");
    let first = source_path("scenes/main.zscene");
    let second = source_path("scenes/other.zscene");

    store
        .write_snapshot(&document, &first, 1, &extension("zscene"), b"first")
        .unwrap();
    assert!(matches!(
        store.write_snapshot(&document, &second, 2, &extension("zscene"), b"second"),
        Err(AutosaveError::RecoverySourceConflict { .. })
    ));
    remove_temporary_root(&root);
}

#[test]
fn recovery_catalog_allows_only_one_concurrent_first_source_mapping() {
    let root = temporary_root("concurrent-source-conflict");
    let store = AutosaveStore::new(&root);
    let start = Arc::new(Barrier::new(2));
    let first_store = store.clone();
    let first_start = Arc::clone(&start);
    let first = thread::spawn(move || {
        first_start.wait();
        first_store.write_snapshot(
            &document("scene_main"),
            &source_path("scenes/first.zscene"),
            1,
            &extension("zscene"),
            b"first",
        )
    });
    let second_start = Arc::clone(&start);
    let second = thread::spawn(move || {
        second_start.wait();
        store.write_snapshot(
            &document("scene_main"),
            &source_path("scenes/second.zscene"),
            2,
            &extension("zscene"),
            b"second",
        )
    });

    let results = [first.join().unwrap(), second.join().unwrap()];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(AutosaveError::RecoverySourceConflict { .. })))
            .count(),
        1
    );
    remove_temporary_root(&root);
}

#[test]
fn recovery_catalog_fails_closed_when_snapshot_metadata_is_missing() {
    let root = temporary_root("missing-metadata");
    let directory = root.join(".zircon/autosave/scene_main");
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("1.zscene"), b"orphaned snapshot").unwrap();
    let store = AutosaveStore::new(&root);

    assert!(matches!(
        store.recovery_candidates(),
        Err(AutosaveError::RecoveryMetadataMissing { .. })
    ));
    remove_temporary_root(&root);
}

#[test]
fn recovery_catalog_keeps_a_missing_source_as_an_explicit_restore_candidate() {
    let root = temporary_root("missing-source");
    let store = AutosaveStore::new(&root);
    let document = document("scene_main");
    let source = source_path("scenes/missing.zscene");

    store
        .write_snapshot(&document, &source, 1, &extension("zscene"), b"snapshot")
        .unwrap();

    let candidates = store.recovery_candidates().unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0].source_path(),
        root.join(source.as_path()).as_path()
    );
    assert!(candidates[0].should_offer_recovery());
    remove_temporary_root(&root);
}

#[test]
fn recovery_catalog_rejects_duplicate_numeric_snapshot_sequences() {
    let root = temporary_root("duplicate-recovery-sequence");
    let store = AutosaveStore::new(&root);
    let document = document("scene_main");
    let source = source_path("scenes/main.zscene");

    store
        .write_snapshot(&document, &source, 1, &extension("zscene"), b"snapshot")
        .unwrap();
    fs::write(
        root.join(".zircon/autosave/scene_main/1.backup"),
        b"conflicting snapshot",
    )
    .unwrap();

    assert!(matches!(
        store.recovery_candidates(),
        Err(AutosaveError::DuplicateRecoverySequence { .. })
    ));
    remove_temporary_root(&root);
}

fn temporary_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-editor-recovery-catalog-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos()
    ))
}

fn remove_temporary_root(path: &std::path::Path) {
    let _ = fs::remove_dir_all(path);
}
