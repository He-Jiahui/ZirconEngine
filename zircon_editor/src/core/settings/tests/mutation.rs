use super::*;

struct RecordingPersistenceHealthSubscriber {
    observed: Sender<SettingsPersistenceHealthSnapshot>,
}

impl SettingsPersistenceHealthSubscriber for RecordingPersistenceHealthSubscriber {
    fn persistence_health_changed(&self, snapshot: &SettingsPersistenceHealthSnapshot) {
        let _ = self.observed.send(*snapshot);
    }
}

#[test]
fn coordinator_rejects_an_unbound_project_before_mutating_authority() {
    let coordinator = SettingsMutationCoordinator::in_memory_with_defaults();
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let before_generation = coordinator.authority().snapshot().generation();
    let before = coordinator.authority().resolved_setting(&snap_key).unwrap();

    let error = coordinator
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(2.5))
        .unwrap_err();

    assert_eq!(error, SettingsMutationError::ProjectNotBound);
    assert_eq!(
        coordinator.authority().snapshot().generation(),
        before_generation
    );
    let after = coordinator.authority().resolved_setting(&snap_key).unwrap();
    assert_eq!(after.value(), before.value());
}

#[test]
fn coordinator_rejects_an_unwritable_user_source_before_mutating_authority() {
    let coordinator = SettingsMutationCoordinator::in_memory_with_defaults();
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let before_generation = coordinator.authority().snapshot().generation();

    let error = coordinator
        .set(SettingsScope::User, &snap_key, SettingValue::Float(2.5))
        .unwrap_err();

    assert_eq!(error, SettingsMutationError::UserSourceUnavailable);
    assert_eq!(
        coordinator.authority().snapshot().generation(),
        before_generation
    );
}

#[test]
fn coordinator_keeps_an_invalid_project_source_read_only() {
    let root = temporary_root("mutation-invalid-project");
    let project_settings = root.join(".zircon").join("settings.toml");
    fs::create_dir_all(project_settings.parent().unwrap()).unwrap();
    fs::write(&project_settings, "retired = true\n").unwrap();
    let coordinator = SettingsMutationCoordinator::in_memory_with_defaults();

    let binding = coordinator.bind_project(&root).unwrap();
    assert!(matches!(
        binding.load(),
        SettingsProjectLayerLoad::Invalid { .. }
    ));
    let error = coordinator
        .set(
            SettingsScope::Project,
            &key(VIEWPORT_TRANSLATE_STEP_KEY),
            SettingValue::Float(2.5),
        )
        .unwrap_err();

    assert_eq!(error, SettingsMutationError::ProjectSourceInvalid);
    assert_eq!(
        fs::read_to_string(project_settings).unwrap(),
        "retired = true\n"
    );
    remove_temporary_root(&root);
}

#[test]
fn coordinator_persists_a_typed_user_mutation_through_its_shutdown_fence() {
    let root = temporary_root("mutation-user-persistence");
    let store = SettingsStore::from_roots(&root, None);
    let authority = Arc::new(SettingsAuthority::with_defaults());
    let persistence = SettingsPersistenceService::new(
        Arc::clone(&authority),
        crate::core::jobs::test_job_scheduler(),
    );
    let coordinator =
        SettingsMutationCoordinator::new(Arc::clone(&authority), persistence, Some(store.clone()));
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);

    let receipt = coordinator
        .set(SettingsScope::User, &snap_key, SettingValue::Float(2.5))
        .unwrap();
    assert_eq!(
        receipt.disposition(),
        SettingsMutationDisposition::PersistentQueued
    );
    assert_eq!(
        receipt.authority_generation(),
        authority.snapshot().generation()
    );
    assert_eq!(receipt.persistence_generation().unwrap().get(), 1);
    coordinator.flush_then_shutdown().unwrap().finish().unwrap();

    let mut restored = settings_registry_with_defaults();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut restored),
        Ok(SettingsLoad::Loaded { .. })
    ));
    assert_eq!(restored.resolve(&snap_key), Some(&SettingValue::Float(2.5)));
    remove_temporary_root(&root);
}

#[test]
fn coordinator_publishes_durable_health_from_worker_terminal() {
    let root = temporary_root("mutation-user-health");
    let store = SettingsStore::from_roots(&root, None);
    let authority = Arc::new(SettingsAuthority::with_defaults());
    let persistence = SettingsPersistenceService::new(
        Arc::clone(&authority),
        crate::core::jobs::test_job_scheduler(),
    );
    let coordinator =
        SettingsMutationCoordinator::new(Arc::clone(&authority), persistence, Some(store));
    let (observed_tx, observed_rx) = mpsc::channel();
    coordinator.configure_persistence_health_subscriber(Arc::new(
        RecordingPersistenceHealthSubscriber {
            observed: observed_tx,
        },
    ));

    let receipt = coordinator
        .set(
            SettingsScope::User,
            &key(VIEWPORT_TRANSLATE_STEP_KEY),
            SettingValue::Float(2.5),
        )
        .unwrap();
    coordinator.flush_then_shutdown().unwrap().finish().unwrap();

    let health = coordinator.persistence_health_snapshot().user();
    assert_eq!(health.file_generation(), receipt.persistence_generation());
    assert_eq!(health.status(), SettingsPersistenceHealthStatus::Durable);
    assert!(observed_rx.try_iter().any(|snapshot| {
        snapshot.user().file_generation() == receipt.persistence_generation()
            && snapshot.user().status() == SettingsPersistenceHealthStatus::Durable
    }));
    remove_temporary_root(&root);
}

#[test]
fn coordinator_clear_coalesces_the_latest_user_document_state() {
    let root = temporary_root("mutation-user-clear");
    let store = SettingsStore::from_roots(&root, None);
    let authority = Arc::new(SettingsAuthority::with_defaults());
    let persistence = SettingsPersistenceService::new(
        Arc::clone(&authority),
        crate::core::jobs::test_job_scheduler(),
    );
    let coordinator =
        SettingsMutationCoordinator::new(Arc::clone(&authority), persistence, Some(store.clone()));
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);

    coordinator
        .set(SettingsScope::User, &snap_key, SettingValue::Float(8.0))
        .unwrap();
    let cleared = coordinator.clear(SettingsScope::User, &snap_key).unwrap();
    assert_eq!(
        cleared.disposition(),
        SettingsMutationDisposition::PersistentQueued
    );
    coordinator.flush_then_shutdown().unwrap().finish().unwrap();

    let mut restored = settings_registry_with_defaults();
    store.load_into(SettingsScope::User, &mut restored).unwrap();
    assert_eq!(
        restored.resolve(&snap_key),
        settings_registry_with_defaults().resolve(&snap_key)
    );
    remove_temporary_root(&root);
}

#[test]
fn coordinator_projects_deferred_admission_as_retryable_health() {
    let root = temporary_root("mutation-admission-retry");
    let store = SettingsStore::from_roots(&root, None);
    let authority = Arc::new(SettingsAuthority::with_defaults());
    let persistence = SettingsPersistenceService::with_limits(
        Arc::clone(&authority),
        crate::core::jobs::test_job_scheduler(),
        SettingsPersistenceLimits {
            max_entries: 1,
            max_retained_bytes: 1,
        },
    );
    let coordinator =
        SettingsMutationCoordinator::new(Arc::clone(&authority), persistence, Some(store));

    let receipt = coordinator
        .set(
            SettingsScope::User,
            &key(VIEWPORT_TRANSLATE_STEP_KEY),
            SettingValue::Float(3.0),
        )
        .unwrap();
    assert!(matches!(
        receipt.disposition(),
        SettingsMutationDisposition::AppliedPendingAdmission(
            SettingsPersistenceSubmitError::LaneAdmission(_)
        )
    ));
    let health = coordinator.persistence_health_snapshot().user();
    assert_eq!(health.file_generation(), receipt.persistence_generation());
    assert!(matches!(
        health.status(),
        SettingsPersistenceHealthStatus::PendingAdmission(
            SettingsPersistenceSubmitError::LaneAdmission(_)
        )
    ));
    assert!(health.status().is_retryable());
    let retry = coordinator.retry_pending(SettingsScope::User).unwrap();
    assert_eq!(retry.file_generation(), receipt.persistence_generation());
    assert!(matches!(
        retry.disposition(),
        SettingsPersistenceRetryDisposition::PendingAdmission(
            SettingsPersistenceSubmitError::LaneAdmission(_)
        )
    ));
    assert!(matches!(
        coordinator.flush_then_shutdown(),
        Err(SettingsPersistenceSubmitError::LaneAdmission(_))
    ));

    coordinator.shutdown().wait();
    remove_temporary_root(&root);
}

#[test]
fn coordinator_rejects_retry_for_the_session_layer() {
    let coordinator = SettingsMutationCoordinator::in_memory_with_defaults();

    assert_eq!(
        coordinator.retry_pending(SettingsScope::Session),
        Err(SettingsMutationError::NonPersistentScope(
            SettingsScope::Session
        ))
    );
}
