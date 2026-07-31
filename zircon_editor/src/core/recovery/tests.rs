use std::collections::BTreeSet;
use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use super::{
    AutosaveDocumentId, AutosaveDocumentState, AutosaveExtension, AutosaveJobPolicy,
    AutosavePolicy, AutosaveScheduler, AutosaveStore,
};
use crate::core::jobs::{JobCategory, JobPriority, MutexGroup};

fn document_id(value: &str) -> AutosaveDocumentId {
    AutosaveDocumentId::parse(value).expect("test document id should be valid")
}

fn extension(value: &str) -> AutosaveExtension {
    AutosaveExtension::parse(value).expect("test extension should be valid")
}

#[test]
fn autosave_requires_elapsed_interval_and_a_dirty_document() {
    let policy = AutosavePolicy::new(Duration::from_secs(10)).unwrap();
    let mut scheduler = AutosaveScheduler::new(policy);
    let document = document_id("scene_main");
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        true,
    )];
    let clean = [AutosaveDocumentState::from_dirty_for_test(
        document.clone(),
        false,
    )];

    assert!(scheduler.plan(Duration::from_secs(9), &dirty).is_none());
    assert!(scheduler.plan(Duration::from_secs(10), &clean).is_none());

    let plan = scheduler
        .plan(Duration::from_secs(10), &dirty)
        .expect("a due dirty document must create an autosave plan");
    assert_eq!(plan.documents(), &[document.clone()]);
    assert!(scheduler.is_in_flight());
    assert!(scheduler.plan(Duration::from_secs(11), &dirty).is_none());

    scheduler.mark_finished(Duration::from_secs(10));
    assert!(!scheduler.is_in_flight());
    assert!(scheduler.plan(Duration::from_secs(19), &dirty).is_none());
    assert!(scheduler.plan(Duration::from_secs(20), &dirty).is_some());
}

#[test]
fn autosave_execution_failure_releases_the_in_flight_plan_on_the_normal_interval() {
    let policy = AutosavePolicy::new(Duration::from_secs(10)).unwrap();
    let mut scheduler = AutosaveScheduler::new(policy);
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document_id("scene_main"),
        true,
    )];

    assert!(scheduler.plan(Duration::from_secs(10), &dirty).is_some());
    scheduler.mark_finished(Duration::from_secs(12));
    assert!(!scheduler.is_in_flight());
    assert!(scheduler.plan(Duration::from_secs(21), &dirty).is_none());
    assert!(scheduler.plan(Duration::from_secs(22), &dirty).is_some());
}

#[test]
fn autosave_submission_failure_releases_the_in_flight_plan_for_retry() {
    let policy = AutosavePolicy::new(Duration::from_secs(10)).unwrap();
    let mut scheduler = AutosaveScheduler::new(policy);
    let dirty = [AutosaveDocumentState::from_dirty_for_test(
        document_id("scene_main"),
        true,
    )];

    assert!(scheduler.plan(Duration::from_secs(10), &dirty).is_some());
    assert!(scheduler.is_in_flight());
    scheduler.mark_submission_failed();
    assert!(!scheduler.is_in_flight());
    assert!(scheduler.plan(Duration::from_secs(10), &dirty).is_some());
}

#[test]
fn autosave_dirty_inputs_are_projected_from_editor03_history_state() {
    let source = include_str!("autosave.rs");

    assert!(source.contains("pub fn from_history_dirty"));
    assert!(source.contains("HistoryDirtyState"));
    assert!(!source.contains("pub fn dirty("));
    assert!(!source.contains("pub fn clean("));
}

#[test]
fn autosave_job_policy_uses_background_misc_and_the_save_mutex_group() {
    let save_mutex = MutexGroup::parse("save_scene_main").unwrap();
    let policy = AutosaveJobPolicy::for_save_mutex(save_mutex);

    assert_eq!(policy.category(), JobCategory::Misc);
    assert_eq!(policy.priority(), JobPriority::Background);
    assert_eq!(policy.save_mutex_group().as_str(), "save_scene_main");

    let _job_spec = policy.build_job_spec(&document_id("scene_main"));
}

#[test]
fn autosave_writes_only_the_project_autosave_tree_and_preserves_source_bytes() {
    let root = temporary_root("source-guard");
    let source_path = root.join("scenes").join("main.zscene");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, b"authoritative scene source").unwrap();
    let before = fs::read(&source_path).unwrap();
    let store = AutosaveStore::new(&root);

    let snapshot = store
        .write_snapshot(
            &document_id("scene_main"),
            1,
            &extension("zscene"),
            b"autosave snapshot",
        )
        .unwrap();

    assert_eq!(fs::read(&source_path).unwrap(), before);
    assert_eq!(fs::read(&snapshot).unwrap(), b"autosave snapshot");
    assert_eq!(
        snapshot,
        root.join(".zircon")
            .join("autosave")
            .join("scene_main")
            .join("1.zscene")
    );
    remove_temporary_root(&root);
}

#[test]
fn autosave_rotates_each_document_to_the_latest_three_sequences() {
    let root = temporary_root("rotation");
    let store = AutosaveStore::new(&root);
    let document = document_id("world_01");
    let zscene_extension = extension("zscene");

    for sequence in 1..=4 {
        store
            .write_snapshot(
                &document,
                sequence,
                &zscene_extension,
                format!("snapshot-{sequence}").as_bytes(),
            )
            .unwrap();
    }

    let directory = root.join(".zircon").join("autosave").join("world_01");
    let names = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        names,
        BTreeSet::from([
            "2.zscene".to_string(),
            "3.zscene".to_string(),
            "4.zscene".to_string(),
        ])
    );
    remove_temporary_root(&root);
}

#[test]
fn autosave_rejects_reusing_an_existing_snapshot_sequence() {
    let root = temporary_root("duplicate-sequence");
    let store = AutosaveStore::new(&root);
    let document = document_id("scene_main");
    let zscene_extension = extension("zscene");
    let backup_extension = extension("backup");

    store
        .write_snapshot(&document, 1, &zscene_extension, b"first snapshot")
        .unwrap();
    assert!(store
        .write_snapshot(&document, 1, &zscene_extension, b"replacement snapshot")
        .is_err());
    assert!(store
        .write_snapshot(
            &document,
            1,
            &backup_extension,
            b"different-extension snapshot",
        )
        .is_err());
    assert_eq!(
        fs::read(root.join(".zircon/autosave/scene_main/1.zscene")).unwrap(),
        b"first snapshot"
    );
    remove_temporary_root(&root);
}

#[test]
fn autosave_sequence_reservation_allows_only_one_concurrent_extension() {
    let root = temporary_root("concurrent-sequence");
    let store = AutosaveStore::new(&root);
    let start = Arc::new(Barrier::new(2));
    let first_store = store.clone();
    let first_start = Arc::clone(&start);
    let first = thread::spawn(move || {
        first_start.wait();
        first_store.write_snapshot(
            &document_id("scene_main"),
            1,
            &extension("zscene"),
            b"zscene snapshot",
        )
    });
    let second_store = store;
    let second_start = Arc::clone(&start);
    let second = thread::spawn(move || {
        second_start.wait();
        second_store.write_snapshot(
            &document_id("scene_main"),
            1,
            &extension("backup"),
            b"backup snapshot",
        )
    });

    let results = [first.join().unwrap(), second.join().unwrap()];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);

    let snapshots = fs::read_dir(root.join(".zircon/autosave/scene_main"))
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<BTreeSet<_>>();
    assert_eq!(snapshots.len(), 1);
    assert!(snapshots.iter().all(|name| !name.starts_with('.')));
    remove_temporary_root(&root);
}

#[test]
fn autosave_rejects_path_traversal_identifiers_and_extensions() {
    assert!(AutosaveDocumentId::parse("../source").is_err());
    assert!(AutosaveDocumentId::parse("scene/main").is_err());
    assert!(AutosaveDocumentId::parse(" ").is_err());
    assert!(AutosaveExtension::parse("../source").is_err());
    assert!(AutosaveExtension::parse("scene.data").is_err());
    assert!(AutosaveExtension::parse(" ").is_err());
}

fn temporary_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-editor-autosave-{label}-{}-{}",
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
