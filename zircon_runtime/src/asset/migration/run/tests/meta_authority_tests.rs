use std::fs;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Barrier};
use std::thread;
use std::time::Duration;

use crate::asset::project::{
    lock_meta_document_path, AssetMetaDocument, AssetMetaPreviewStateCasResult,
    AssetMetaPreviewStateExpectation, PreviewState,
};
use crate::asset::{AssetKind, AssetUri, AssetUuid};
use crate::core::resource::io::atomic_write;

use super::{
    migrate_project_assets, migrate_project_assets_with_commit_hook, AssetMigrationMode,
    AssetMigrationOptions, CommitFault,
};

#[test]
fn asset_meta_preview_state_migration_reads_after_the_active_writer_releases() {
    let (root, path, mut current) = migration_fixture();
    let active = lock_meta_document_path(&path).unwrap();
    let start = Arc::new(Barrier::new(2));
    let worker_start = Arc::clone(&start);
    let worker_root = root.clone();
    let (done_send, done_receive) = mpsc::channel();
    let worker = thread::spawn(move || {
        worker_start.wait();
        done_send
            .send(migrate_project_assets(AssetMigrationOptions::new(
                worker_root,
                AssetMigrationMode::Apply,
            )))
            .unwrap();
    });

    start.wait();
    let early = done_receive.recv_timeout(Duration::from_millis(100));
    current.preview_state = PreviewState::Ready;
    current.tags.insert("latest-writer".to_string());
    current.import_settings.insert(
        "quality".to_string(),
        toml::Value::String("high".to_string()),
    );
    // Simulate the active writer publishing while it owns the path authority.
    atomic_write(&path, &current.to_pretty_bytes().unwrap()).unwrap();
    drop(active);
    let was_blocked = matches!(early, Err(mpsc::RecvTimeoutError::Timeout));
    let report = early
        .unwrap_or_else(|_| done_receive.recv_timeout(Duration::from_secs(10)).unwrap())
        .unwrap();
    worker.join().unwrap();
    assert!(
        was_blocked,
        "migration must wait before preparing sidecar bytes"
    );
    assert!(report.succeeded(), "{report:?}");
    assert!(report.changed_files().is_empty());
    assert_eq!(AssetMetaDocument::load(&path).unwrap(), current);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn asset_meta_preview_state_cas_waits_until_migration_publishes() {
    let (root, path, current) = migration_fixture();
    let expected = AssetMetaPreviewStateExpectation::from_document(&current);
    let (prepared_send, prepared_receive) = mpsc::channel();
    let (release_send, release_receive) = mpsc::channel();
    let worker_root = root.clone();
    let migration = thread::spawn(move || {
        migrate_project_assets_with_commit_hook(
            AssetMigrationOptions::new(worker_root, AssetMigrationMode::Apply),
            CommitFault::Never,
            || {
                prepared_send.send(()).unwrap();
                release_receive
                    .recv_timeout(Duration::from_secs(10))
                    .unwrap();
            },
        )
        .unwrap()
    });
    prepared_receive
        .recv_timeout(Duration::from_secs(10))
        .unwrap();

    let (preview_send, preview_receive) = mpsc::channel();
    let start = Arc::new(Barrier::new(2));
    let preview_start = Arc::clone(&start);
    let preview_path = path.clone();
    let preview = thread::spawn(move || {
        preview_start.wait();
        preview_send
            .send(AssetMetaDocument::compare_and_set_preview_state(
                &preview_path,
                &expected,
                PreviewState::Ready,
            ))
            .unwrap();
    });
    start.wait();
    let early = preview_receive.recv_timeout(Duration::from_millis(100));
    release_send.send(()).unwrap();
    let report = migration.join().unwrap();
    let was_blocked = matches!(early, Err(mpsc::RecvTimeoutError::Timeout));
    let result = early
        .unwrap_or_else(|_| {
            preview_receive
                .recv_timeout(Duration::from_secs(10))
                .unwrap()
        })
        .unwrap();
    preview.join().unwrap();

    assert!(
        was_blocked,
        "preview CAS must wait through migration publication"
    );
    assert!(report.succeeded(), "{report:?}");
    assert_eq!(report.changed_files().len(), 1);
    assert_eq!(
        result,
        AssetMetaPreviewStateCasResult::Updated {
            previous: PreviewState::Dirty,
            current: PreviewState::Ready,
        }
    );
    let mut persisted = AssetMetaDocument::load(&path).unwrap();
    assert_eq!(persisted.preview_state, PreviewState::Ready);
    persisted.preview_state = current.preview_state;
    assert_eq!(persisted, current);
    fs::remove_dir_all(root).unwrap();
}

fn migration_fixture() -> (PathBuf, PathBuf, AssetMetaDocument) {
    let root = std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
        .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("target"))
        .join("zircon-test-output")
        .join(format!("migration-meta-authority-{}", AssetUuid::new()));
    fs::create_dir_all(root.join("assets")).unwrap();
    fs::write(
        root.join("zircon-project.toml"),
        "name = \"Migration CAS Fixture\"\nformat_version = 3\nproject_guid = \"cb3f4863-28f3-4ef7-bd0d-c38c9189a8a1\"\ndefault_scene = \"res://scenes/main.scene.toml\"\nasset_roots = [\"assets\"]\nlibrary_version = 1\n",
    )
    .unwrap();
    fs::write(root.join("assets/hero.data"), b"fixture source").unwrap();
    let path = root.join("assets/hero.data.zmeta");
    let mut current = AssetMetaDocument::new(
        AssetUuid::new(),
        AssetUri::parse("res://hero.data").unwrap(),
        AssetKind::Data,
    );
    current.source_digest = "migration-source".to_string();
    let mut legacy: toml::Value =
        toml::from_str(std::str::from_utf8(&current.to_pretty_bytes().unwrap()).unwrap()).unwrap();
    let table = legacy.as_table_mut().unwrap();
    table.insert("format_version".to_string(), toml::Value::Integer(6));
    let digest = table.remove("source_digest").unwrap();
    table.insert("source_hash".to_string(), digest);
    fs::write(&path, toml::to_string_pretty(&legacy).unwrap()).unwrap();
    (root, path, current)
}
