use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;

static NEXT_TEST_OUTPUT_ID: AtomicU64 = AtomicU64::new(1);

#[test]
fn astra_m12_migration_preserves_artifact_identity_exhaustion() {
    let error = map_transaction_error(DurableTransactionError::ArtifactIdentityExhausted(
        crate::core::resource::io::ArtifactIdentityExhausted,
    ));
    assert!(matches!(
        error,
        AssetMigrationError::ArtifactIdentityExhausted(_)
    ));
    assert_eq!(
        error.to_string(),
        "durable I/O artifact identity space is exhausted"
    );
}

#[test]
fn migration_reports_an_unsynced_commit_point_as_pending_recovery() {
    let output_root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"));
    let root = output_root.join("zircon-test-output").join(format!(
        "zircon-migration-commit-point-sync-{}-{}",
        std::process::id(),
        NEXT_TEST_OUTPUT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let target = root.join("asset.zmeta");
    fs::create_dir_all(&root).unwrap();
    fs::write(&target, b"old-generation").unwrap();

    let error = apply_transaction(
        &root,
        vec![PendingDocument {
            path: target.clone(),
            bytes: b"new-generation".to_vec(),
            reference_count: 0,
            retired_path: None,
        }],
        CommitFault::FailCommitPointSync,
    )
    .expect_err("migration must not report an unresolved commit marker as durable apply");

    assert!(error.to_string().contains("durability is unresolved"));
    assert_eq!(fs::read(&target).unwrap(), b"new-generation");
    assert_eq!(
        fs::read_dir(root.join(".zircon").join(JOURNAL_DIRECTORY))
            .unwrap()
            .count(),
        1
    );
    fs::remove_dir_all(root).unwrap();
}
