use std::thread;

use super::*;
use crate::core::asset::import_flow::state::{
    ImportFinishAction, ImportFlowState, ImportGenerationKey, ImportReservation, ReserveAttempt,
};

#[test]
fn admission_waiter_observes_original_fast_failure() {
    let jobs = test_job_system();
    let backend = Arc::new(RecordingBackend::default());
    backend.fail.store(true, Ordering::SeqCst);
    let index = index_for("res://textures/admission-race.png");
    let entered = Arc::new((Mutex::new(false), Condvar::new()));
    let observer_entered = Arc::new((Mutex::new(false), Condvar::new()));
    let released = Arc::new((Mutex::new(false), Condvar::new()));
    let hook_once = Arc::new(AtomicBool::new(false));
    let hook = {
        let entered = Arc::clone(&entered);
        let released = Arc::clone(&released);
        let hook_once = Arc::clone(&hook_once);
        Arc::new(move || {
            if hook_once.swap(true, Ordering::SeqCst) {
                return;
            }
            let (entered_lock, entered_changed) = &*entered;
            *entered_lock
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
            entered_changed.notify_all();
            let (released_lock, released_changed) = &*released;
            let mut released = released_lock
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            while !*released {
                released = released_changed
                    .wait(released)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
        }) as Arc<dyn Fn() + Send + Sync>
    };
    let observer_hook = {
        let observer_entered = Arc::clone(&observer_entered);
        Arc::new(move || {
            let (entered, changed) = &*observer_entered;
            *entered
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
            changed.notify_all();
        }) as Arc<dyn Fn() + Send + Sync>
    };
    let flow = EditorAssetImportFlow::with_backend(jobs, backend.clone(), index)
        .with_before_job_submit(hook)
        .with_before_wait_admission(observer_hook);
    let request = EditorAssetImportRequest::new(
        uri("res://textures/admission-race.png"),
        EditorAssetImportReason::Watch,
    );
    let first_flow = flow.clone();
    let first_request = request.clone();
    let first = thread::spawn(move || first_flow.submit(first_request).unwrap());

    let (entered_lock, entered_changed) = &*entered;
    let mut has_entered = entered_lock
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    while !*has_entered {
        has_entered = entered_changed
            .wait(has_entered)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
    drop(has_entered);

    let second_flow = flow.clone();
    let second = thread::spawn(move || second_flow.submit(request).unwrap());
    let (observer_lock, observer_changed) = &*observer_entered;
    let mut observer_is_waiting = observer_lock
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    while !*observer_is_waiting {
        observer_is_waiting = observer_changed
            .wait(observer_is_waiting)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
    drop(observer_is_waiting);
    let (released_lock, released_changed) = &*released;
    *released_lock
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
    released_changed.notify_all();

    let first = first.join().unwrap();
    let second = second.join().unwrap();
    assert_eq!(first.id(), second.id());
    assert!(matches!(first.wait(), Err(JobError::Failed(_))));
    assert!(matches!(second.wait(), Err(JobError::Failed(_))));
    assert_eq!(
        backend
            .calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        1
    );
}

#[test]
fn registry_generation_change_retries_before_job_submission() {
    let jobs = test_job_system();
    let backend = Arc::new(RecordingBackend::default());
    let target = uri("res://textures/revalidate.png");
    let index = index_for("res://textures/revalidate.png");
    let hook_once = Arc::new(AtomicBool::new(false));
    let hook = {
        let hook_once = Arc::clone(&hook_once);
        let index = Arc::clone(&index);
        let target = target.clone();
        Arc::new(move || {
            if hook_once.swap(true, Ordering::SeqCst) {
                return;
            }
            let registry = AssetRegistryIndex::from_entries(vec![AssetRegistryEntry::new(
                uuid("asset"),
                target.clone(),
                AssetKind::Texture,
                "digest-after-revalidate",
            )])
            .unwrap();
            index
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .replace_runtime_registry(Arc::new(registry));
        }) as Arc<dyn Fn() + Send + Sync>
    };
    let flow = EditorAssetImportFlow::with_backend(jobs, backend.clone(), index.clone())
        .with_before_generation_validate(hook);

    flow.submit(EditorAssetImportRequest::new(
        target.clone(),
        EditorAssetImportReason::DigestMismatch,
    ))
    .unwrap()
    .wait()
    .unwrap();

    assert_eq!(
        backend
            .calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        1
    );
    assert_eq!(import_state(&index, &target), EditorAssetImportState::Stale);
}

#[test]
fn uuid_import_lifecycle_blocks_start_and_stale_clear_boundaries() {
    let limits = EditorAssetImportAdmissionLimits::new(8, usize::MAX, Duration::from_secs(60));
    let uuid = uuid("lifecycle");
    let first_key = ImportGenerationKey::new(
        uuid,
        Arc::new(uri("res://textures/lifecycle-a.png")),
        Arc::from("digest-a"),
    );
    let second_key = ImportGenerationKey::new(
        uuid,
        Arc::new(uri("res://textures/lifecycle-b.png")),
        Arc::from("digest-b"),
    );
    let third_key = ImportGenerationKey::new(
        uuid,
        Arc::new(uri("res://textures/lifecycle-c.png")),
        Arc::from("digest-c"),
    );
    let now = Instant::now();
    let mut state = ImportFlowState::default();
    let (first_identity, token) = match state
        .reserve(
            first_key.clone(),
            EditorAssetImportReason::Watch,
            now,
            limits,
        )
        .unwrap()
    {
        ReserveAttempt::Ready(ImportReservation::New {
            flight_identity,
            begin_uuid: Some(token),
            ..
        }) => (flight_identity, token),
        _ => panic!("first UUID generation must own the starting token"),
    };
    assert!(matches!(
        state
            .reserve(
                second_key.clone(),
                EditorAssetImportReason::Watch,
                now,
                limits,
            )
            .unwrap(),
        ReserveAttempt::UuidTransitionPending
    ));
    assert!(state.mark_uuid_ready(token));
    let second_identity = match state
        .reserve(
            second_key.clone(),
            EditorAssetImportReason::Watch,
            now,
            limits,
        )
        .unwrap()
    {
        ReserveAttempt::Ready(ImportReservation::New {
            flight_identity,
            begin_uuid: None,
            ..
        }) => flight_identity,
        _ => panic!("ready UUID lifecycle must admit a serialized generation"),
    };
    assert_eq!(
        state.finish(&first_key, first_identity, true, 0, limits, now),
        ImportFinishAction::NoIndexTransition
    );
    assert_eq!(
        state.finish(&second_key, second_identity, true, 0, limits, now),
        ImportFinishAction::ClearUuid(token)
    );
    assert!(matches!(
        state
            .reserve(
                third_key.clone(),
                EditorAssetImportReason::Watch,
                now,
                limits,
            )
            .unwrap(),
        ReserveAttempt::UuidTransitionPending
    ));
    assert!(state.complete_uuid_clear(token));
    let successor = match state
        .reserve(third_key, EditorAssetImportReason::Watch, now, limits)
        .unwrap()
    {
        ReserveAttempt::Ready(ImportReservation::New {
            begin_uuid: Some(token),
            ..
        }) => token,
        _ => panic!("post-clear generation must own a successor UUID token"),
    };
    assert_ne!(successor, token);
    assert!(!state.complete_uuid_clear(token));
}

#[test]
fn completed_generation_expires_even_under_hot_key_reuse() {
    let backend = Arc::new(RecordingBackend::default());
    let target = uri("res://textures/completed-ttl.png");
    let flow = EditorAssetImportFlow::with_backend_and_limits(
        test_job_system(),
        backend.clone(),
        index_for("res://textures/completed-ttl.png"),
        EditorAssetImportAdmissionLimits::new(4, usize::MAX, Duration::ZERO),
    );

    let first = flow
        .submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::Watch,
        ))
        .unwrap();
    let first_id = first.id();
    first.wait().unwrap();
    let second = flow
        .submit(EditorAssetImportRequest::new(
            target,
            EditorAssetImportReason::Manual,
        ))
        .unwrap();
    let second_id = second.id();
    second.wait().unwrap();

    assert_ne!(first_id, second_id);
    assert_eq!(
        backend
            .calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        2
    );
}

#[test]
fn completed_result_bytes_are_reclaimed_before_new_admission() {
    let backend = Arc::new(RecordingBackend::default());
    let target = uri("res://textures/oversized-result.png");
    let mut oversized = imported_status(&target);
    oversized.source_hash = "x".repeat(8 * 1024);
    *backend
        .status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(oversized);
    let flow = EditorAssetImportFlow::with_backend_and_limits(
        test_job_system(),
        backend.clone(),
        index_for("res://textures/oversized-result.png"),
        EditorAssetImportAdmissionLimits::new(4, 2 * 1024, Duration::from_secs(60)),
    );

    let first = flow
        .submit(EditorAssetImportRequest::new(
            target.clone(),
            EditorAssetImportReason::Watch,
        ))
        .unwrap();
    let first_id = first.id();
    first.wait().unwrap();
    let second = flow
        .submit(EditorAssetImportRequest::new(
            target,
            EditorAssetImportReason::Manual,
        ))
        .unwrap();
    let second_id = second.id();
    second.wait().unwrap();

    assert_ne!(first_id, second_id);
    assert_eq!(
        backend
            .calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        2
    );
}
