use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;

use crate::core::recovery::{
    AutosaveContentDigest, AutosaveDocumentId, AutosaveError, AutosaveExtension,
    AutosaveRecoveryCatalogDiagnosticKind, AutosaveSnapshotProvenance, AutosaveSourceDigest,
    AutosaveSourcePath, AutosaveStore, RestoreFreshness,
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

fn snapshot_provenance() -> AutosaveSnapshotProvenance {
    AutosaveSnapshotProvenance::capture(0, AutosaveSourceDigest::missing())
}

fn source_provenance(source: &[u8], generation: u64) -> AutosaveSnapshotProvenance {
    AutosaveSnapshotProvenance::capture(
        generation,
        AutosaveSourceDigest::Present(AutosaveContentDigest::from_bytes(source)),
    )
}

#[test]
fn recovery_source_path_rejects_absolute_and_traversal_paths() {
    assert!(matches!(
        AutosaveSourcePath::parse("../outside.zscene"),
        Err(AutosaveError::InvalidRecoverySourcePath { .. })
    ));
    assert!(matches!(
        AutosaveSourcePath::parse(
            std::env::current_dir()
                .expect("current directory should be available")
                .join("outside.zscene")
        ),
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
        .write_snapshot(
            &document,
            &source,
            1,
            &extension("zscene"),
            &source_provenance(b"authoritative source", 7),
            b"first",
        )
        .unwrap();
    let latest = store
        .write_snapshot(
            &document,
            &source,
            2,
            &extension("zscene"),
            &source_provenance(b"authoritative source", 8),
            b"second",
        )
        .unwrap();

    let report = store.recovery_catalog().unwrap();
    let candidates = report.candidates();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].document(), &document);
    assert_eq!(candidates[0].source_path(), source_file.as_path());
    assert_eq!(candidates[0].autosave_path(), latest.as_path());
    assert_eq!(
        candidates[0].freshness(),
        RestoreFreshness::SnapshotAheadOfSource
    );
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
        .write_snapshot(
            &document,
            &first,
            1,
            &extension("zscene"),
            &snapshot_provenance(),
            b"first",
        )
        .unwrap();
    assert!(matches!(
        store.write_snapshot(
            &document,
            &second,
            2,
            &extension("zscene"),
            &snapshot_provenance(),
            b"second",
        ),
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
            &snapshot_provenance(),
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
            &snapshot_provenance(),
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
fn recovery_catalog_quarantines_missing_metadata_and_keeps_other_documents() {
    let root = temporary_root("missing-metadata");
    let malformed_directory = root.join(".zircon/autosave/malformed_scene");
    fs::create_dir_all(&malformed_directory).unwrap();
    fs::write(malformed_directory.join("1.zscene"), b"orphaned snapshot").unwrap();
    let store = AutosaveStore::new(&root);
    let valid_document = document("scene_main");
    let valid_source = source_path("scenes/main.zscene");
    store
        .write_snapshot(
            &valid_document,
            &valid_source,
            1,
            &extension("zscene"),
            &snapshot_provenance(),
            b"valid snapshot",
        )
        .unwrap();

    let report = store.recovery_catalog().unwrap();
    assert_eq!(report.candidates().len(), 1);
    assert_eq!(report.candidates()[0].document(), &valid_document);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].path(),
        malformed_directory.as_path()
    );
    assert!(matches!(
        report.diagnostics()[0].kind(),
        AutosaveRecoveryCatalogDiagnosticKind::MetadataMissing
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
        .write_snapshot(
            &document,
            &source,
            1,
            &extension("zscene"),
            &snapshot_provenance(),
            b"snapshot",
        )
        .unwrap();

    let report = store.recovery_catalog().unwrap();
    let candidates = report.candidates();
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0].source_path(),
        root.join(source.as_path()).as_path()
    );
    assert!(candidates[0].should_offer_recovery());
    assert_eq!(candidates[0].freshness(), RestoreFreshness::SourceMissing);
    remove_temporary_root(&root);
}

#[test]
fn recovery_catalog_marks_an_externally_changed_source_as_diverged_without_mtime() {
    let root = temporary_root("source-diverged");
    let source = source_path("scenes/main.zscene");
    let source_file = root.join(source.as_path());
    fs::create_dir_all(source_file.parent().unwrap()).unwrap();
    fs::write(&source_file, b"base source").unwrap();
    let store = AutosaveStore::new(&root);
    let document = document("scene_main");

    store
        .write_snapshot(
            &document,
            &source,
            1,
            &extension("zscene"),
            &source_provenance(b"base source", 12),
            b"autosave edits",
        )
        .unwrap();
    fs::write(&source_file, b"external edits").unwrap();

    let report = store.recovery_catalog().unwrap();
    assert!(matches!(
        report.candidates(),
        [candidate] if candidate.freshness() == RestoreFreshness::SourceDiverged
    ));
    remove_temporary_root(&root);
}

#[test]
fn recovery_catalog_omits_a_snapshot_already_committed_to_the_source() {
    let root = temporary_root("snapshot-already-committed");
    let source = source_path("scenes/main.zscene");
    let source_file = root.join(source.as_path());
    fs::create_dir_all(source_file.parent().unwrap()).unwrap();
    fs::write(&source_file, b"base source").unwrap();
    let store = AutosaveStore::new(&root);
    let document = document("scene_main");

    store
        .write_snapshot(
            &document,
            &source,
            1,
            &extension("zscene"),
            &source_provenance(b"base source", 12),
            b"autosave edits",
        )
        .unwrap();
    fs::write(&source_file, b"autosave edits").unwrap();

    let report = store.recovery_catalog().unwrap();
    assert!(report.candidates().is_empty());
    assert!(report.diagnostics().is_empty());
    remove_temporary_root(&root);
}

#[test]
fn recovery_catalog_quarantines_a_committed_checksum_mismatch() {
    let root = temporary_root("committed-checksum-mismatch");
    let store = AutosaveStore::new(&root);
    let document = document("scene_main");
    let source = source_path("scenes/main.zscene");

    store
        .write_snapshot(
            &document,
            &source,
            1,
            &extension("zscene"),
            &snapshot_provenance(),
            b"snapshot",
        )
        .unwrap();
    fs::write(
        root.join(".zircon/autosave/scene_main/1.zscene"),
        b"tampered",
    )
    .unwrap();

    let report = store.recovery_catalog().unwrap();
    assert!(report.candidates().is_empty());
    assert_eq!(report.diagnostics().len(), 1);
    assert!(matches!(
        report.diagnostics()[0].kind(),
        AutosaveRecoveryCatalogDiagnosticKind::CommittedChecksumMismatch
    ));
    remove_temporary_root(&root);
}

fn temporary_root(label: &str) -> std::path::PathBuf {
    std::env::current_dir()
        .expect("current directory should be available")
        .join("target")
        .join(format!(
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
