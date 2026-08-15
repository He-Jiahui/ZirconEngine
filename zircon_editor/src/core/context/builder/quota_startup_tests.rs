use std::fs;
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::core::jobs::{
    EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, EDITOR_JOB_EXPORT_QUOTA_KEY,
};
use crate::core::settings::{
    SettingValue, SettingsKey, SettingsScope, SettingsStore, SettingsUserLayerLoad,
};

use super::EditorContextBuilder;

struct QuotaGateJob {
    started: mpsc::Sender<()>,
    release: mpsc::Receiver<()>,
}

impl EditorJob for QuotaGateJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        self.started.send(()).unwrap();
        self.release.recv_timeout(Duration::from_secs(5)).unwrap();
        Ok(())
    }
}

#[test]
fn restart_only_job_quota_changes_apply_to_the_next_context_admission_owner() {
    let root = std::env::temp_dir().join(format!(
        "zircon-editor-context-quota-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let store = SettingsStore::from_roots(&root, None);
    let scheduler = crate::core::jobs::test_job_scheduler();
    let context_a = EditorContextBuilder::new(scheduler.clone())
        .with_settings_store(store.clone())
        .build();
    assert!(matches!(
        context_a.settings().user_layer_load(),
        Some(SettingsUserLayerLoad::Missing { .. })
    ));

    let export_key = SettingsKey::parse(EDITOR_JOB_EXPORT_QUOTA_KEY).unwrap();
    let change = context_a
        .settings()
        .set(SettingsScope::User, &export_key, SettingValue::Int(2))
        .unwrap()
        .unwrap();
    assert!(change.requires_restart);
    store
        .save_authority_layer(SettingsScope::User, context_a.settings())
        .unwrap();

    let (first_started, first_started_rx) = mpsc::channel();
    let (first_release, first_release_rx) = mpsc::channel();
    let (second_started, second_started_rx) = mpsc::channel();
    let (second_release, second_release_rx) = mpsc::channel();
    let first = context_a
        .jobs()
        .submit(
            EditorJobSpec::new("context A export one", JobCategory::Export),
            QuotaGateJob {
                started: first_started,
                release: first_release_rx,
            },
        )
        .unwrap();
    let second = context_a
        .jobs()
        .submit(
            EditorJobSpec::new("context A export two", JobCategory::Export),
            QuotaGateJob {
                started: second_started,
                release: second_release_rx,
            },
        )
        .unwrap();
    assert_eq!(context_a.jobs().admission_snapshot().pending_entries(), 1);
    first_release.send(()).unwrap();
    second_release.send(()).unwrap();
    first_started_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    second_started_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert!(first.wait().is_ok());
    assert!(second.wait().is_ok());

    let context_b = EditorContextBuilder::new(scheduler)
        .with_settings_store(store.clone())
        .build();
    assert!(matches!(
        context_b.settings().user_layer_load(),
        Some(SettingsUserLayerLoad::Loaded { .. })
    ));
    let (first_started, first_started_rx) = mpsc::channel();
    let (first_release, first_release_rx) = mpsc::channel();
    let (second_started, second_started_rx) = mpsc::channel();
    let (second_release, second_release_rx) = mpsc::channel();
    let first = context_b
        .jobs()
        .submit(
            EditorJobSpec::new("context B export one", JobCategory::Export),
            QuotaGateJob {
                started: first_started,
                release: first_release_rx,
            },
        )
        .unwrap();
    let second = context_b
        .jobs()
        .submit(
            EditorJobSpec::new("context B export two", JobCategory::Export),
            QuotaGateJob {
                started: second_started,
                release: second_release_rx,
            },
        )
        .unwrap();
    assert_eq!(context_b.jobs().admission_snapshot().pending_entries(), 0);
    first_release.send(()).unwrap();
    second_release.send(()).unwrap();
    first_started_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    second_started_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert!(first.wait().is_ok());
    assert!(second.wait().is_ok());

    let _ = fs::remove_dir_all(root);
}
