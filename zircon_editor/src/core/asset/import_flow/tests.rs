use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use zircon_runtime::asset::registry::{AssetRegistryEntry, AssetRegistryIndex};
use zircon_runtime::asset::{AssetKind, AssetStatusRecord, AssetUri, AssetUuid};
use zircon_runtime::core::CoreError;

use crate::core::asset::{EditorAssetImportState, EditorAssetIndex};
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};
use crate::core::jobs::{
    EditorJobLimits, JobCategory, JobError, JobEventKind, JobEventPumpBudget, JobSubmitError,
    test_job_system, test_job_system_with_bus, test_job_system_with_limits,
};

#[path = "tests/concurrency.rs"]
mod concurrency;
use super::{
    AssetImportBackend, EditorAssetImportAdmissionLimits, EditorAssetImportFlow,
    EditorAssetImportReason, EditorAssetImportRequest, EditorAssetImportSubmitError,
};

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

fn uuid(label: &str) -> AssetUuid {
    AssetUuid::from_stable_label(label)
}

fn index_for(path: &str) -> Arc<Mutex<EditorAssetIndex>> {
    index_for_assets(&[("asset", path, "digest")])
}

fn index_for_assets(entries: &[(&str, &str, &str)]) -> Arc<Mutex<EditorAssetIndex>> {
    let registry = AssetRegistryIndex::from_entries(
        entries
            .iter()
            .map(|(label, path, digest)| {
                AssetRegistryEntry::new(uuid(label), uri(path), AssetKind::Texture, *digest)
            })
            .collect::<Vec<_>>(),
    )
    .unwrap();
    Arc::new(Mutex::new(EditorAssetIndex::new(Arc::new(registry))))
}

fn import_state(index: &Arc<Mutex<EditorAssetIndex>>, path: &AssetUri) -> EditorAssetImportState {
    index
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .row_by_path(path)
        .unwrap()
        .import_state()
}

#[derive(Default)]
struct RecordingBackend {
    calls: Mutex<Vec<AssetUri>>,
    fail: AtomicBool,
    status: Mutex<Option<AssetStatusRecord>>,
}

impl AssetImportBackend for RecordingBackend {
    fn import(&self, uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(uri.clone());
        if self.fail.load(Ordering::SeqCst) {
            Err(CoreError::Initialization(
                "asset import".to_owned(),
                "planned failure".to_owned(),
            ))
        } else {
            Ok(self
                .status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone())
        }
    }
}

fn imported_status(path: &AssetUri) -> AssetStatusRecord {
    AssetStatusRecord {
        id: uuid("asset").to_string(),
        uri: path.to_string(),
        kind: AssetKind::Texture,
        artifact_uri: Some("lib://derived/texture.zasset".to_owned()),
        imported: true,
        source_hash: "digest".to_owned(),
        importer_id: "texture".to_owned(),
        importer_version: 1,
        config_hash: "settings".to_owned(),
    }
}

#[test]
fn successful_import_uses_runtime_backend_and_clears_importing_state() {
    let jobs = test_job_system();
    let backend = Arc::new(RecordingBackend::default());
    let index = index_for("res://textures/sky.png");
    let flow = EditorAssetImportFlow::with_backend(jobs, backend.clone(), index.clone());
    let target = uri("res://textures/sky.png");
    let expected_status = imported_status(&target);
    *backend
        .status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(expected_status.clone());

    let result = flow
        .submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::Manual,
        ))
        .unwrap()
        .wait()
        .unwrap();

    assert_eq!(result.uri(), &target);
    assert_eq!(result.reasons(), vec![EditorAssetImportReason::Manual]);
    assert_eq!(result.status(), Some(&expected_status));
    assert_eq!(
        *backend
            .calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec![target.clone()]
    );
    assert_eq!(import_state(&index, &target), EditorAssetImportState::Stale);
}

#[test]
fn backend_failure_is_typed_and_clears_importing_state() {
    let jobs = test_job_system();
    let backend = Arc::new(RecordingBackend::default());
    backend.fail.store(true, Ordering::SeqCst);
    let index = index_for("res://textures/broken.png");
    let flow = EditorAssetImportFlow::with_backend(jobs, backend, index.clone());
    let target = uri("res://textures/broken.png");

    let error = flow
        .submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::DigestMismatch,
        ))
        .unwrap()
        .wait()
        .unwrap_err();

    assert!(matches!(error, JobError::Failed(_)));
    assert!(matches!(
        error.downcast_ref::<CoreError>(),
        Some(CoreError::Initialization(owner, message))
            if owner == "asset import" && message == "planned failure"
    ));
    assert_eq!(import_state(&index, &target), EditorAssetImportState::Stale);
}

#[test]
fn failed_generation_can_be_submitted_again() {
    let jobs = test_job_system();
    let backend = Arc::new(RecordingBackend::default());
    backend.fail.store(true, Ordering::SeqCst);
    let target = uri("res://textures/retry.png");
    let index = index_for("res://textures/retry.png");
    let flow = EditorAssetImportFlow::with_backend(jobs, backend.clone(), index);

    assert!(
        flow.submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::Watch,
        ))
        .unwrap()
        .wait()
        .is_err()
    );
    backend.fail.store(false, Ordering::SeqCst);
    flow.submit(EditorAssetImportRequest::new(
        target.clone(),
        EditorAssetImportReason::Manual,
    ))
    .unwrap()
    .wait()
    .unwrap();

    assert_eq!(
        *backend
            .calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec![target.clone(), target]
    );
}

#[test]
fn unknown_uri_is_rejected_before_job_submission() {
    let jobs = test_job_system();
    let backend = Arc::new(RecordingBackend::default());
    let index = index_for("res://textures/indexed.png");
    let flow = EditorAssetImportFlow::with_backend(jobs, backend.clone(), index);
    let missing = uri("res://textures/missing.png");

    let error = flow
        .submit(EditorAssetImportRequest::new(
            missing.clone(),
            EditorAssetImportReason::Watch,
        ))
        .unwrap_err();

    assert_eq!(
        error,
        EditorAssetImportSubmitError::AssetNotIndexed { uri: missing }
    );
    assert!(
        backend
            .calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty()
    );
}

#[test]
fn duplicate_generation_storm_shares_one_job_and_merges_reasons() {
    let jobs = test_job_system();
    let (started_tx, started_rx) = mpsc::channel();
    let backend = Arc::new(BlockingBackend {
        calls: AtomicUsize::new(0),
        started: Mutex::new(Some(started_tx)),
        released: (Mutex::new(false), Condvar::new()),
    });
    let index = index_for("res://models/ship.glb");
    let flow = EditorAssetImportFlow::with_backend(jobs, backend.clone(), index);
    let target = uri("res://models/ship.glb");
    let first = flow
        .submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::Manual,
        ))
        .unwrap();
    let shared_id = first.id();
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut tickets = vec![first];
    for index in 0..10_000 {
        let reason = match index % 3 {
            0 => EditorAssetImportReason::Watch,
            1 => EditorAssetImportReason::DigestMismatch,
            _ => EditorAssetImportReason::Manual,
        };
        let ticket = flow
            .submit(EditorAssetImportRequest::new(target.clone(), reason))
            .unwrap();
        assert_eq!(ticket.id(), shared_id);
        tickets.push(ticket);
    }

    backend.release();
    for ticket in tickets {
        let result = ticket.wait().unwrap();
        assert_eq!(
            result.reasons(),
            vec![
                EditorAssetImportReason::Watch,
                EditorAssetImportReason::DigestMismatch,
                EditorAssetImportReason::Manual,
            ]
        );
    }

    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
}

struct PathMigrationBackend {
    old_started: Mutex<Option<mpsc::Sender<()>>>,
    old_released: (Mutex<bool>, Condvar),
    new_released: (Mutex<bool>, Condvar),
}

impl PathMigrationBackend {
    fn release(gate: &(Mutex<bool>, Condvar)) {
        let (released, changed) = gate;
        *released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
        changed.notify_all();
    }

    fn wait(gate: &(Mutex<bool>, Condvar)) {
        let (released, changed) = gate;
        let mut released = released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !*released {
            released = changed
                .wait(released)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }
}

impl AssetImportBackend for PathMigrationBackend {
    fn import(&self, uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError> {
        if uri.path() == "textures/old.png" {
            if let Some(started) = self
                .old_started
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take()
            {
                let _ = started.send(());
            }
            Self::wait(&self.old_released);
        } else {
            Self::wait(&self.new_released);
        }
        Ok(None)
    }
}

#[test]
fn uuid_importing_survives_registry_path_migration_until_all_uri_jobs_finish() {
    let jobs = test_job_system();
    let (old_started_tx, old_started_rx) = mpsc::channel();
    let backend = Arc::new(PathMigrationBackend {
        old_started: Mutex::new(Some(old_started_tx)),
        old_released: (Mutex::new(false), Condvar::new()),
        new_released: (Mutex::new(false), Condvar::new()),
    });
    let old_uri = uri("res://textures/old.png");
    let new_uri = uri("res://textures/new.png");
    let index = index_for("res://textures/old.png");
    let flow = EditorAssetImportFlow::with_backend(jobs, backend.clone(), index.clone());

    let old = flow
        .submit(EditorAssetImportRequest::new(
            old_uri,
            EditorAssetImportReason::Watch,
        ))
        .unwrap();
    old_started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let migrated_registry = AssetRegistryIndex::from_entries(vec![AssetRegistryEntry::new(
        uuid("asset"),
        new_uri.clone(),
        AssetKind::Texture,
        "digest",
    )])
    .unwrap();
    index
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .replace_runtime_registry(Arc::new(migrated_registry));
    let new = flow
        .submit(EditorAssetImportRequest::new(
            new_uri.clone(),
            EditorAssetImportReason::Watch,
        ))
        .unwrap();

    PathMigrationBackend::release(&backend.old_released);
    old.wait().unwrap();
    assert_eq!(
        import_state(&index, &new_uri),
        EditorAssetImportState::Importing
    );

    PathMigrationBackend::release(&backend.new_released);
    new.wait().unwrap();
    assert_eq!(
        import_state(&index, &new_uri),
        EditorAssetImportState::Stale
    );
}

#[test]
fn import_job_publishes_zero_to_one_progress_sequence() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::default());
    let backend = Arc::new(RecordingBackend::default());
    let index = index_for("res://textures/progress.png");
    let flow = EditorAssetImportFlow::with_backend(jobs.clone(), backend, index);

    flow.submit(EditorAssetImportRequest::new(
        uri("res://textures/progress.png"),
        EditorAssetImportReason::Manual,
    ))
    .unwrap()
    .wait()
    .unwrap();
    jobs.pump_events_with_budget(JobEventPumpBudget::new(usize::MAX, Duration::from_secs(1)));

    let progress = bus
        .drain_deliveries(subscriber)
        .into_iter()
        .filter_map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::Job(event) => match event.kind() {
                JobEventKind::Progress {
                    completed,
                    total,
                    message,
                } => Some((*completed, *total, message.clone())),
                _ => None,
            },
            payload => panic!("unexpected payload: {payload:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        progress,
        vec![
            (0, 1, "Importing res://textures/progress.png".to_owned()),
            (1, 1, "Imported res://textures/progress.png".to_owned()),
        ]
    );
}

struct PanicBackend;

impl AssetImportBackend for PanicBackend {
    fn import(&self, _uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError> {
        panic!("planned import backend panic")
    }
}

#[test]
fn backend_panic_releases_import_lifecycle() {
    let jobs = test_job_system();
    let target = uri("res://textures/panic.png");
    let index = index_for("res://textures/panic.png");
    let flow = EditorAssetImportFlow::with_backend(jobs, Arc::new(PanicBackend), index.clone());

    let error = flow
        .submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::Manual,
        ))
        .unwrap()
        .wait()
        .unwrap_err();

    assert!(matches!(error, JobError::Panicked(_)));
    assert_eq!(import_state(&index, &target), EditorAssetImportState::Stale);
}

#[test]
fn shutdown_submission_rejection_releases_import_lifecycle() {
    let jobs = test_job_system();
    assert!(jobs.shutdown(Instant::now()).is_empty());
    let target = uri("res://textures/shutdown.png");
    let index = index_for("res://textures/shutdown.png");
    let flow = EditorAssetImportFlow::with_backend(
        jobs,
        Arc::new(RecordingBackend::default()),
        index.clone(),
    );

    let error = flow
        .submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::Manual,
        ))
        .unwrap_err();

    assert_eq!(
        error,
        EditorAssetImportSubmitError::Job(JobSubmitError::ShuttingDown)
    );
    assert_eq!(import_state(&index, &target), EditorAssetImportState::Stale);
}

struct BlockingBackend {
    calls: AtomicUsize,
    started: Mutex<Option<mpsc::Sender<()>>>,
    released: (Mutex<bool>, Condvar),
}

impl BlockingBackend {
    fn release(&self) {
        let (released, changed) = &self.released;
        *released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
        changed.notify_all();
    }
}

impl AssetImportBackend for BlockingBackend {
    fn import(&self, _uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(started) = self
            .started
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            let _ = started.send(());
        }
        let (released, changed) = &self.released;
        let mut released = released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !*released {
            released = changed
                .wait(released)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        Ok(None)
    }
}

#[test]
fn shared_flight_cancel_releases_importing_once() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Import, 1));
    let (started_tx, started_rx) = mpsc::channel();
    let backend = Arc::new(BlockingBackend {
        calls: AtomicUsize::new(0),
        started: Mutex::new(Some(started_tx)),
        released: (Mutex::new(false), Condvar::new()),
    });
    let target = uri("res://models/blocked.glb");
    let index = index_for("res://models/blocked.glb");
    let flow = EditorAssetImportFlow::with_backend(jobs.clone(), backend.clone(), index.clone());
    let request = EditorAssetImportRequest::new(target.clone(), EditorAssetImportReason::Manual);

    let first = flow.submit(request.clone()).unwrap();
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let second = flow.submit(request).unwrap();
    assert_eq!(first.id(), second.id());
    assert!(jobs.cancel(second.id()));
    assert_eq!(
        import_state(&index, &target),
        EditorAssetImportState::Importing
    );

    backend.release();
    assert!(matches!(first.wait(), Err(JobError::Cancelled)));
    assert!(matches!(second.wait(), Err(JobError::Cancelled)));
    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
    assert_eq!(import_state(&index, &target), EditorAssetImportState::Stale);
}

#[test]
fn admission_limits_bound_entries_bytes_and_oldest_age() {
    let count_jobs = test_job_system();
    let (count_started_tx, count_started_rx) = mpsc::channel();
    let count_backend = Arc::new(BlockingBackend {
        calls: AtomicUsize::new(0),
        started: Mutex::new(Some(count_started_tx)),
        released: (Mutex::new(false), Condvar::new()),
    });
    let count_index = index_for_assets(&[
        ("first", "res://textures/first.png", "digest-first"),
        ("second", "res://textures/second.png", "digest-second"),
    ]);
    let count_flow = EditorAssetImportFlow::with_backend_and_limits(
        count_jobs,
        count_backend.clone(),
        count_index,
        EditorAssetImportAdmissionLimits::new(1, usize::MAX, Duration::from_secs(60)),
    );
    let first = count_flow
        .submit(EditorAssetImportRequest::new(
            uri("res://textures/first.png"),
            EditorAssetImportReason::Watch,
        ))
        .unwrap();
    count_started_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert!(matches!(
        count_flow.submit(EditorAssetImportRequest::new(
            uri("res://textures/second.png"),
            EditorAssetImportReason::Watch,
        )),
        Err(EditorAssetImportSubmitError::FlightLimitReached { limit: 1 })
    ));
    count_backend.release();
    first.wait().unwrap();

    let byte_flow = EditorAssetImportFlow::with_backend_and_limits(
        test_job_system(),
        Arc::new(RecordingBackend::default()),
        index_for("res://textures/bytes.png"),
        EditorAssetImportAdmissionLimits::new(1, 1, Duration::from_secs(60)),
    );
    assert!(matches!(
        byte_flow.submit(EditorAssetImportRequest::new(
            uri("res://textures/bytes.png"),
            EditorAssetImportReason::Manual,
        )),
        Err(EditorAssetImportSubmitError::ByteLimitExceeded { limit: 1, .. })
    ));

    let age_jobs = test_job_system();
    let (age_started_tx, age_started_rx) = mpsc::channel();
    let age_backend = Arc::new(BlockingBackend {
        calls: AtomicUsize::new(0),
        started: Mutex::new(Some(age_started_tx)),
        released: (Mutex::new(false), Condvar::new()),
    });
    let age_index = index_for_assets(&[
        ("oldest", "res://textures/oldest.png", "digest-oldest"),
        ("incoming", "res://textures/incoming.png", "digest-incoming"),
    ]);
    let age_flow = EditorAssetImportFlow::with_backend_and_limits(
        age_jobs,
        age_backend.clone(),
        age_index,
        EditorAssetImportAdmissionLimits::new(2, usize::MAX, Duration::ZERO),
    );
    let oldest = age_flow
        .submit(EditorAssetImportRequest::new(
            uri("res://textures/oldest.png"),
            EditorAssetImportReason::Watch,
        ))
        .unwrap();
    age_started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(
        age_flow.submit(EditorAssetImportRequest::new(
            uri("res://textures/incoming.png"),
            EditorAssetImportReason::Watch,
        )),
        Err(EditorAssetImportSubmitError::OldestFlightAgeExceeded { .. })
    ));
    age_backend.release();
    oldest.wait().unwrap();
}
