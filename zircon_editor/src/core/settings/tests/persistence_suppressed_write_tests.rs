use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use zircon_runtime::core::runtime::tasks::{
    BoundedKeyedIoWaitResult, JobScheduler, TaskPool, TaskPoolDescriptor,
};

use super::*;
use crate::core::settings::{SettingValue, SettingsProjectLayerLoad, VIEWPORT_TRANSLATE_STEP_KEY};

#[test]
fn project_ticket_without_active_binding_reports_skipped_stale_instead_of_written() {
    let root = temporary_root("unbound");
    let project_root = root.join("project");
    let store = SettingsStore::from_roots(root.join("user"), Some(&project_root));
    let authority = Arc::new(SettingsAuthority::with_defaults());
    let change = project_change(&authority);
    let service =
        SettingsPersistenceService::new(authority, crate::core::jobs::test_job_scheduler());

    let ticket = service.submit(&change, store.clone()).unwrap();
    assert!(matches!(
        ticket.wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(_)
    ));
    assert_eq!(
        ticket.persistence_terminal(),
        Some(SettingsPersistenceTerminal::SkippedStale)
    );
    assert!(!store.paths().project().unwrap().exists());
    remove_temporary_root(&root);
}

#[test]
fn stale_project_store_submitted_after_switch_cannot_report_written() {
    let root = temporary_root("stale-project");
    let project_a = root.join("project-a");
    let project_b = root.join("project-b");
    let store_a = SettingsStore::from_roots(root.join("user"), Some(&project_a));
    let store_b = SettingsStore::from_roots(root.join("user"), Some(&project_b));
    let authority = Arc::new(SettingsAuthority::with_defaults());
    assert!(matches!(
        authority.load_project_layer_from_store(&store_a),
        SettingsProjectLayerLoad::Missing { .. }
    ));
    let change = project_change(&authority);
    authority.clear_project_layer();
    assert!(matches!(
        authority.load_project_layer_from_store(&store_b),
        SettingsProjectLayerLoad::Missing { .. }
    ));
    let service =
        SettingsPersistenceService::new(authority, crate::core::jobs::test_job_scheduler());

    let ticket = service.submit(&change, store_a.clone()).unwrap();
    assert!(matches!(
        ticket.wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(_)
    ));
    assert_eq!(
        ticket.persistence_terminal(),
        Some(SettingsPersistenceTerminal::SkippedStale)
    );
    assert!(!store_a.paths().project().unwrap().exists());
    assert!(!store_b.paths().project().unwrap().exists());
    remove_temporary_root(&root);
}

#[test]
fn queued_project_write_cannot_report_written_after_project_switch() {
    let root = temporary_root("queued-switch");
    let project_a = root.join("project-a");
    let project_b = root.join("project-b");
    let store_a = SettingsStore::from_roots(root.join("user"), Some(&project_a));
    let store_b = SettingsStore::from_roots(root.join("user"), Some(&project_b));
    let authority = Arc::new(SettingsAuthority::with_defaults());
    assert!(matches!(
        authority.load_project_layer_from_store(&store_a),
        SettingsProjectLayerLoad::Missing { .. }
    ));
    let change = project_change(&authority);

    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let scheduler = JobScheduler::from_pool(pool);
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    scheduler.spawn(move || {
        started_sender.send(()).unwrap();
        release_receiver.recv().unwrap();
    });
    started_receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("the sole worker must hold the queue before submission");

    let service = SettingsPersistenceService::new(Arc::clone(&authority), scheduler);
    let ticket = service.submit(&change, store_a.clone()).unwrap();
    authority.clear_project_layer();
    assert!(matches!(
        authority.load_project_layer_from_store(&store_b),
        SettingsProjectLayerLoad::Missing { .. }
    ));
    release_sender.send(()).unwrap();

    assert_eq!(
        ticket.wait_for_persistence_until(Instant::now() + Duration::from_secs(5)),
        SettingsPersistenceWaitResult::Terminal(SettingsPersistenceTerminal::SkippedStale)
    );
    assert!(!store_a.paths().project().unwrap().exists());
    assert!(!store_b.paths().project().unwrap().exists());
    remove_temporary_root(&root);
}

#[test]
fn invalid_project_source_never_reports_written_or_replaces_the_source() {
    let root = temporary_root("invalid-project");
    let project_root = root.join("project");
    let store = SettingsStore::from_roots(root.join("user"), Some(&project_root));
    let settings_path = store.paths().project().unwrap().to_path_buf();
    fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
    let invalid_source = b"retired = true\n";
    fs::write(&settings_path, invalid_source).unwrap();
    let authority = Arc::new(SettingsAuthority::with_defaults());
    assert!(matches!(
        authority.load_project_layer_from_store(&store),
        SettingsProjectLayerLoad::Invalid { .. }
    ));
    let change = project_change(&authority);
    let service =
        SettingsPersistenceService::new(authority, crate::core::jobs::test_job_scheduler());
    let (sender, receiver) = mpsc::channel();
    let ticket = service
        .submit_observed(
            &change,
            service.allocate_file_generation().unwrap(),
            store,
            move |terminal| {
                sender.send(terminal).unwrap();
            },
        )
        .unwrap();

    assert!(matches!(
        ticket.wait_until(Instant::now() + Duration::from_secs(5)),
        BoundedKeyedIoWaitResult::Terminal(_)
    ));
    assert_eq!(
        ticket.persistence_terminal(),
        Some(SettingsPersistenceTerminal::BlockedInvalid)
    );
    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(5)).unwrap(),
        SettingsPersistenceTerminal::BlockedInvalid
    );
    assert_eq!(fs::read(&settings_path).unwrap(), invalid_source);
    remove_temporary_root(&root);
}

fn project_change(authority: &SettingsAuthority) -> SettingChange {
    let key = SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
    authority
        .set(SettingsScope::Project, &key, SettingValue::Float(2.5))
        .unwrap()
        .expect("the project setting must change")
}

fn temporary_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zircon-editor-265-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn remove_temporary_root(root: &Path) {
    assert!(root.starts_with(std::env::temp_dir()));
    fs::remove_dir_all(root).unwrap();
}
