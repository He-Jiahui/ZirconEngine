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
    assert_eq!(restored.resolve(&snap_key), Ok(&SettingValue::Float(2.5)));
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

fn guarded_string_coordinator(
    scope: SettingsScope,
    store: Option<SettingsStore>,
) -> (SettingsMutationCoordinator, SettingsKey) {
    let mut registry = settings_registry_with_defaults();
    let string_key = key("editor.test.guarded_string");
    registry
        .register(
            SettingDefinition::new(
                string_key.clone(),
                scope,
                SettingSchema::String { maximum_bytes: 32 },
                SettingValue::String("default".to_owned()),
                true,
                presentation(
                    "settings.test.string.label",
                    "settings.test.string.description",
                    &["settings.category.editor"],
                ),
            )
            .unwrap(),
        )
        .unwrap();
    let authority = Arc::new(SettingsAuthority::from_registry_for_performance_benchmark(
        registry,
    ));
    let persistence = SettingsPersistenceService::new(
        Arc::clone(&authority),
        crate::core::jobs::test_job_scheduler(),
    );
    (
        SettingsMutationCoordinator::new(authority, persistence, store),
        string_key,
    )
}

#[test]
fn guarded_preferences_string_and_reset_keep_persistence_and_restart_receipts() {
    let mut fixture =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_string_persistence").unwrap();
    let empty_document = write_versioned_text(&SettingsDocument {
        values: BTreeMap::new(),
    })
    .unwrap();
    fixture
        .create_file("settings.toml", empty_document.as_bytes())
        .unwrap();
    let store = SettingsStore::from_roots(fixture.root(), None);
    let (coordinator, string_key) =
        guarded_string_coordinator(SettingsScope::User, Some(store.clone()));
    let initial = coordinator.capture_values([&string_key]).unwrap();
    let receipt = coordinator
        .set_from_context(
            &initial,
            SettingsScope::User,
            &string_key,
            SettingValue::String("中文 value".to_owned()),
        )
        .unwrap();
    assert!(receipt.requires_restart());
    assert_eq!(
        receipt.disposition(),
        SettingsMutationDisposition::PersistentQueued
    );
    assert!(receipt.persistence_generation().is_some());
    let displayed = coordinator.capture_values([&string_key]).unwrap();
    let reset = coordinator
        .clear_from_context(&displayed, SettingsScope::User, &string_key)
        .unwrap()
        .unwrap();
    assert!(reset.requires_restart());
    assert_eq!(
        reset.disposition(),
        SettingsMutationDisposition::PersistentQueued
    );
    let empty = coordinator.capture_values([&string_key]).unwrap();
    assert_eq!(
        coordinator.clear_from_context(&empty, SettingsScope::User, &string_key),
        Ok(None)
    );
    coordinator.flush_then_shutdown().unwrap().finish().unwrap();
    assert_eq!(
        coordinator
            .authority()
            .resolved_setting(&string_key)
            .unwrap()
            .source(),
        SettingValueSource::Default
    );
    let mut restored = SettingsRegistry::default();
    // The final durable document contains no override for the cleared test setting.
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut restored),
        Ok(SettingsLoad::Loaded { .. })
    ));
    fixture
        .expect_written_bytes("settings.toml", empty_document.as_bytes())
        .unwrap();
}

#[test]
fn guarded_preferences_reject_direct_authority_writes_and_release_the_phase() {
    let (coordinator, string_key) = guarded_string_coordinator(SettingsScope::Session, None);
    let initial = coordinator.capture_values([&string_key]).unwrap();
    coordinator
        .authority()
        .set(
            SettingsScope::Session,
            &string_key,
            SettingValue::String("other writer".to_owned()),
        )
        .unwrap();
    let actual = coordinator.authority().snapshot().generation();
    let expected_error = SettingsMutationError::Settings(SettingsError::StaleGeneration {
        expected: initial.generation(),
        actual,
    });
    assert_eq!(
        coordinator.set_from_context(
            &initial,
            SettingsScope::Session,
            &string_key,
            SettingValue::String("stale".to_owned())
        ),
        Err(expected_error)
    );
    // The original batch had no override. Its reset no-op must also reject stale values.
    assert!(matches!(
        coordinator.clear_from_context(&initial, SettingsScope::Session, &string_key),
        Err(SettingsMutationError::Settings(
            SettingsError::StaleGeneration { .. }
        ))
    ));
    let current = coordinator.capture_values([&string_key]).unwrap();
    assert_eq!(
        current.values().values()[0].value(),
        &SettingValue::String("other writer".to_owned())
    );
    assert!(coordinator
        .set_from_context(
            &current,
            SettingsScope::Session,
            &string_key,
            SettingValue::String("current".to_owned())
        )
        .unwrap()
        .changed());
    let displayed_override = coordinator.capture_values([&string_key]).unwrap();
    coordinator
        .authority()
        .set(
            SettingsScope::Session,
            &string_key,
            SettingValue::String("later writer".to_owned()),
        )
        .unwrap();
    assert!(matches!(
        coordinator.clear_from_context(&displayed_override, SettingsScope::Session, &string_key),
        Err(SettingsMutationError::Settings(
            SettingsError::StaleGeneration { .. }
        ))
    ));
    assert_eq!(
        coordinator
            .authority()
            .resolved_setting(&string_key)
            .unwrap()
            .value(),
        &SettingValue::String("later writer".to_owned())
    );
    let latest = coordinator.capture_values([&string_key]).unwrap();
    assert!(coordinator
        .clear_from_context(&latest, SettingsScope::Session, &string_key)
        .unwrap()
        .is_some());
}

#[test]
fn guarded_preferences_rebind_and_unbound_roundtrip_reject_equal_value_generations() {
    let first =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_project_first").unwrap();
    let second =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_project_second").unwrap();
    let (coordinator, string_key) = guarded_string_coordinator(SettingsScope::Project, None);
    let unbound = coordinator.capture_values([&string_key]).unwrap();
    coordinator.bind_project(first.root()).unwrap();
    let bound = coordinator.capture_values([&string_key]).unwrap();
    coordinator.clear_project().unwrap();
    let unbound_again = coordinator.capture_values([&string_key]).unwrap();
    assert_eq!(unbound.generation(), unbound_again.generation());
    assert_eq!(
        coordinator.clear_from_context(&unbound, SettingsScope::Project, &string_key),
        Err(SettingsMutationError::StaleProjectBinding)
    );
    coordinator.bind_project(second.root()).unwrap();
    let rebound = coordinator.capture_values([&string_key]).unwrap();
    assert_eq!(bound.generation(), rebound.generation());
    assert_eq!(
        coordinator.set_from_context(
            &bound,
            SettingsScope::Project,
            &string_key,
            SettingValue::String("wrong project".to_owned())
        ),
        Err(SettingsMutationError::StaleProjectBinding)
    );
    assert!(!coordinator.context_is_current(&bound));
    assert_eq!(
        coordinator.clear_from_context(&rebound, SettingsScope::Project, &string_key),
        Ok(None)
    );
    coordinator.flush_then_shutdown().unwrap().finish().unwrap();
}

#[test]
fn guarded_preferences_empty_reset_needs_no_store_and_validation_errors_release_the_phase() {
    let (coordinator, string_key) = guarded_string_coordinator(SettingsScope::User, None);
    let displayed = coordinator.capture_values([&string_key]).unwrap();
    assert_eq!(
        coordinator.clear_from_context(&displayed, SettingsScope::User, &string_key),
        Ok(None)
    );
    assert!(matches!(
        coordinator.clear_from_context(&displayed, SettingsScope::Project, &string_key),
        Err(SettingsMutationError::Settings(
            SettingsError::ScopeNotAllowed { .. }
        ))
    ));
    assert_eq!(
        coordinator.set_from_context(
            &displayed,
            SettingsScope::User,
            &string_key,
            SettingValue::String("no store".to_owned())
        ),
        Err(SettingsMutationError::UserSourceUnavailable)
    );
    assert!(matches!(
        coordinator.set_from_context(
            &displayed,
            SettingsScope::Session,
            &string_key,
            SettingValue::Bool(true)
        ),
        Err(SettingsMutationError::Settings(
            SettingsError::InvalidValue { .. }
        ))
    ));
    let receipt = coordinator
        .set_from_context(
            &displayed,
            SettingsScope::Session,
            &string_key,
            SettingValue::String("valid".to_owned()),
        )
        .unwrap();
    let current = coordinator.capture_values([&string_key]).unwrap();
    let unchanged = coordinator
        .set_from_context(
            &current,
            SettingsScope::Session,
            &string_key,
            SettingValue::String("valid".to_owned()),
        )
        .unwrap();
    assert!(!unchanged.changed());
    assert_eq!(
        unchanged.authority_generation(),
        receipt.authority_generation()
    );
    let empty = coordinator.capture_values(std::iter::empty()).unwrap();
    assert_eq!(
        coordinator.clear_from_context(&empty, SettingsScope::User, &string_key),
        Err(SettingsMutationError::KeyNotCaptured(string_key))
    );
}

#[test]
fn guarded_preferences_conditional_notifications_release_the_original_authority_lock() {
    struct ReentrantProbe {
        authority: std::sync::Weak<SettingsAuthority>,
        observed: Sender<u64>,
    }
    impl SettingsChangeSubscriber for ReentrantProbe {
        fn settings_changed(
            &self,
            _: &[super::super::SettingChange],
            snapshot: &super::super::SettingsSnapshot,
        ) {
            let authority = self.authority.upgrade().unwrap();
            let changes = authority.changes_since(SettingsChangeCursor::origin());
            assert!(!changes.changes.is_empty());
            self.observed.send(snapshot.generation()).unwrap();
        }
    }
    let (coordinator, string_key) = guarded_string_coordinator(SettingsScope::Session, None);
    let (sender, receiver) = mpsc::channel();
    coordinator
        .authority()
        .configure_change_subscriber(Arc::new(ReentrantProbe {
            authority: Arc::downgrade(coordinator.authority()),
            observed: sender,
        }));
    let displayed = coordinator.capture_values([&string_key]).unwrap();
    let receipt = coordinator
        .set_from_context(
            &displayed,
            SettingsScope::Session,
            &string_key,
            SettingValue::String("changed".to_owned()),
        )
        .unwrap();
    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        receipt.authority_generation()
    );
    let current = coordinator.capture_values([&string_key]).unwrap();
    let receipt = coordinator
        .clear_from_context(&current, SettingsScope::Session, &string_key)
        .unwrap()
        .unwrap();
    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        receipt.authority_generation()
    );
}

#[test]
fn guarded_preferences_keep_full_u64_and_fail_closed_at_exhaustion() {
    let mut registry = settings_registry_with_defaults();
    registry.revision = i64::MAX as u64 + 9;
    let authority = Arc::new(SettingsAuthority::from_registry_for_performance_benchmark(
        registry,
    ));
    let persistence = SettingsPersistenceService::new(
        Arc::clone(&authority),
        crate::core::jobs::test_job_scheduler(),
    );
    let coordinator = SettingsMutationCoordinator::new(Arc::clone(&authority), persistence, None);
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let displayed = coordinator.capture_values([&snap_key]).unwrap();
    assert_eq!(displayed.generation(), i64::MAX as u64 + 9);
    authority
        .set(SettingsScope::Session, &snap_key, SettingValue::Float(2.5))
        .unwrap();
    assert!(matches!(
        coordinator.set_from_context(
            &displayed,
            SettingsScope::Session,
            &snap_key,
            SettingValue::Float(5.0)
        ),
        Err(SettingsMutationError::Settings(
            SettingsError::StaleGeneration { .. }
        ))
    ));

    let mut registry = settings_registry_with_defaults();
    registry.revision = u64::MAX;
    let exhausted = SettingsAuthority::from_registry_for_performance_benchmark(registry);
    assert_eq!(
        exhausted.set_if_generation(
            u64::MAX,
            SettingsScope::Session,
            &snap_key,
            SettingValue::Float(5.0)
        ),
        Err(SettingsError::GenerationExhausted)
    );
    assert_eq!(
        exhausted.clear_if_generation(u64::MAX, SettingsScope::Session, &snap_key),
        Err(SettingsError::GenerationExhausted)
    );
}

#[test]
fn guarded_preferences_capture_and_mutation_reject_an_active_project_transition() {
    struct PhaseProbe {
        coordinator: std::sync::Weak<SettingsMutationCoordinator>,
        context: crate::core::settings::SettingsMutationContext,
        key: SettingsKey,
        observed: Sender<(bool, bool)>,
    }
    impl SettingsChangeSubscriber for PhaseProbe {
        fn settings_changed(
            &self,
            _: &[super::super::SettingChange],
            _: &super::super::SettingsSnapshot,
        ) {
            let coordinator = self.coordinator.upgrade().unwrap();
            let capture_rejected = matches!(
                coordinator.capture_values([&self.key]),
                Err(SettingsMutationError::OperationInProgress { .. })
            );
            let mutation_rejected = matches!(
                coordinator.set_from_context(
                    &self.context,
                    SettingsScope::Session,
                    &self.key,
                    SettingValue::Float(5.0)
                ),
                Err(SettingsMutationError::OperationInProgress { .. })
            );
            self.observed
                .send((capture_rejected, mutation_rejected))
                .unwrap();
        }
    }
    let coordinator = Arc::new(SettingsMutationCoordinator::in_memory_with_defaults());
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    coordinator
        .authority()
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(2.5))
        .unwrap();
    let context = coordinator.capture_values([&snap_key]).unwrap();
    let (sender, receiver) = mpsc::channel();
    coordinator
        .authority()
        .configure_change_subscriber(Arc::new(PhaseProbe {
            coordinator: Arc::downgrade(&coordinator),
            context,
            key: snap_key.clone(),
            observed: sender,
        }));
    coordinator.clear_project().unwrap();
    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(2)).unwrap(),
        (true, true)
    );
    let refreshed = coordinator.capture_values([&snap_key]).unwrap();
    assert!(coordinator
        .set_from_context(
            &refreshed,
            SettingsScope::Session,
            &snap_key,
            SettingValue::Float(5.0)
        )
        .unwrap()
        .changed());
}

#[test]
fn guarded_preferences_pending_admission_and_retry_keep_the_original_change_identity() {
    let fixture =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_settings_admission").unwrap();
    let authority = Arc::new(SettingsAuthority::with_defaults());
    let persistence = SettingsPersistenceService::with_limits(
        Arc::clone(&authority),
        crate::core::jobs::test_job_scheduler(),
        SettingsPersistenceLimits {
            max_entries: 1,
            max_retained_bytes: 1,
        },
    );
    let coordinator = SettingsMutationCoordinator::new(
        Arc::clone(&authority),
        persistence,
        Some(SettingsStore::from_roots(fixture.root(), None)),
    );
    let preview_key = key(crate::core::settings::EDITOR_SHOW_PREVIEW_FEATURES_KEY);
    let displayed = coordinator.capture_values([&preview_key]).unwrap();
    let receipt = coordinator
        .set_from_context(
            &displayed,
            SettingsScope::User,
            &preview_key,
            SettingValue::Bool(true),
        )
        .unwrap();
    assert!(matches!(
        receipt.disposition(),
        SettingsMutationDisposition::AppliedPendingAdmission(
            SettingsPersistenceSubmitError::LaneAdmission(_)
        )
    ));
    let health = coordinator.persistence_health_snapshot();
    assert_eq!(
        health.user().file_generation(),
        receipt.persistence_generation()
    );
    assert!(health.user().status().is_retryable());

    // An old edit must not replace the pending document or its retry identity.
    assert!(matches!(
        coordinator.set_from_context(
            &displayed,
            SettingsScope::User,
            &preview_key,
            SettingValue::Bool(false),
        ),
        Err(SettingsMutationError::Settings(
            SettingsError::StaleGeneration { .. }
        ))
    ));
    assert_eq!(coordinator.persistence_health_snapshot(), health);
    let current = coordinator.capture_values([&preview_key]).unwrap();
    let unchanged = coordinator
        .set_from_context(
            &current,
            SettingsScope::User,
            &preview_key,
            SettingValue::Bool(true),
        )
        .unwrap();
    assert!(!unchanged.changed());
    assert_eq!(unchanged.persistence_generation(), None);
    assert_eq!(coordinator.persistence_health_snapshot(), health);

    let retry = coordinator.retry_pending(SettingsScope::User).unwrap();
    assert_eq!(retry.file_generation(), receipt.persistence_generation());
    assert!(matches!(
        retry.disposition(),
        SettingsPersistenceRetryDisposition::PendingAdmission(
            SettingsPersistenceSubmitError::LaneAdmission(_)
        )
    ));
    assert_eq!(authority.snapshot().generation(), current.generation());
    assert_eq!(
        authority.resolved_setting(&preview_key).unwrap().value(),
        &SettingValue::Bool(true)
    );

    let reset = coordinator
        .clear_from_context(&current, SettingsScope::User, &preview_key)
        .unwrap()
        .unwrap();
    assert!(matches!(
        reset.disposition(),
        SettingsMutationDisposition::AppliedPendingAdmission(
            SettingsPersistenceSubmitError::LaneAdmission(_)
        )
    ));
    assert_ne!(
        reset.persistence_generation(),
        receipt.persistence_generation()
    );
    let reset_health = coordinator.persistence_health_snapshot();
    assert_eq!(
        reset_health.user().file_generation(),
        reset.persistence_generation()
    );
    let inherited = coordinator.capture_values([&preview_key]).unwrap();
    assert_eq!(
        coordinator.clear_from_context(&inherited, SettingsScope::User, &preview_key),
        Ok(None)
    );
    assert_eq!(coordinator.persistence_health_snapshot(), reset_health);
    assert_eq!(
        authority.resolved_setting(&preview_key).unwrap().source(),
        SettingValueSource::Default
    );
    assert!(matches!(
        coordinator.flush_then_shutdown(),
        Err(SettingsPersistenceSubmitError::LaneAdmission(_))
    ));
    coordinator.shutdown().wait();
    assert!(!fixture.root().join("settings.toml").exists());
}

#[test]
fn guarded_preferences_conditional_mutation_cannot_follow_a_concurrent_direct_write() {
    for clear in [false, true] {
        for _ in 0..8 {
            let authority = Arc::new(SettingsAuthority::with_defaults());
            let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
            authority
                .set(SettingsScope::Session, &snap_key, SettingValue::Float(2.5))
                .unwrap();
            let expected = authority.snapshot().generation();
            let barrier = Arc::new(std::sync::Barrier::new(2));
            let writer_authority = Arc::clone(&authority);
            let writer_barrier = Arc::clone(&barrier);
            let writer_key = snap_key.clone();
            let writer = thread::spawn(move || {
                writer_barrier.wait();
                writer_authority
                    .set(
                        SettingsScope::Session,
                        &writer_key,
                        SettingValue::Float(7.5),
                    )
                    .unwrap()
                    .unwrap()
            });
            barrier.wait();
            let guarded = if clear {
                authority.clear_if_generation(expected, SettingsScope::Session, &snap_key)
            } else {
                authority.set_if_generation(
                    expected,
                    SettingsScope::Session,
                    &snap_key,
                    SettingValue::Float(5.0),
                )
            };
            let direct = writer.join().unwrap();
            match guarded {
                Ok(Some(change)) => {
                    // Success is possible only if the guarded mutation linearized first.
                    assert_eq!(change.revision, expected + 1);
                    assert_eq!(direct.revision, expected + 2);
                }
                Err(SettingsError::StaleGeneration {
                    expected: stale,
                    actual,
                }) => {
                    assert_eq!(stale, expected);
                    assert_eq!(actual, expected + 1);
                    assert_eq!(direct.revision, actual);
                }
                unexpected => panic!("unexpected conditional mutation outcome: {unexpected:?}"),
            }
            assert_eq!(authority.snapshot().generation(), direct.revision);
            assert_eq!(
                authority.resolved_setting(&snap_key).unwrap().value(),
                &SettingValue::Float(7.5)
            );
        }
    }
}

#[test]
fn guarded_preferences_context_cannot_cross_coordinators_sharing_one_authority() {
    let authority = Arc::new(SettingsAuthority::with_defaults());
    let coordinator = || {
        SettingsMutationCoordinator::new(
            Arc::clone(&authority),
            SettingsPersistenceService::new(
                Arc::clone(&authority),
                crate::core::jobs::test_job_scheduler(),
            ),
            None,
        )
    };
    let first = coordinator();
    let second = coordinator();
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let foreign = first.capture_values([&snap_key]).unwrap();
    let local = second.capture_values([&snap_key]).unwrap();
    assert_eq!(foreign.values(), local.values());
    assert_ne!(foreign, local);
    assert!(first.context_is_current(&foreign));
    assert!(!second.context_is_current(&foreign));
    assert_eq!(
        second.set_from_context(
            &foreign,
            SettingsScope::Session,
            &snap_key,
            SettingValue::Float(2.5),
        ),
        Err(SettingsMutationError::ForeignMutationContext)
    );
    assert_eq!(
        second.clear_from_context(&foreign, SettingsScope::User, &snap_key),
        Err(SettingsMutationError::ForeignMutationContext)
    );
    assert_eq!(authority.snapshot().generation(), local.generation());

    // Foreign errors release the same operation phase for a subsequent local edit.
    assert!(second
        .set_from_context(
            &local,
            SettingsScope::Session,
            &snap_key,
            SettingValue::Float(5.0),
        )
        .unwrap()
        .changed());
    assert!(!first.context_is_current(&foreign));
    assert!(matches!(
        first.set_from_context(
            &foreign,
            SettingsScope::Session,
            &snap_key,
            SettingValue::Float(2.5),
        ),
        Err(SettingsMutationError::Settings(
            SettingsError::StaleGeneration { .. }
        ))
    ));
    first.flush_then_shutdown().unwrap().finish().unwrap();
    second.flush_then_shutdown().unwrap().finish().unwrap();
}

#[test]
fn guarded_preferences_equal_project_generations_do_not_identify_the_same_source() {
    let first_project =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_context_source_first")
            .unwrap();
    let second_project =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_context_source_second")
            .unwrap();
    let first = SettingsMutationCoordinator::in_memory_with_defaults();
    let second = SettingsMutationCoordinator::in_memory_with_defaults();
    let first_binding = first.bind_project(first_project.root()).unwrap();
    let second_binding = second.bind_project(second_project.root()).unwrap();
    assert_eq!(first_binding.generation(), second_binding.generation());
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let foreign = first.capture_values([&snap_key]).unwrap();
    let local = second.capture_values([&snap_key]).unwrap();
    assert_eq!(foreign.values(), local.values());
    assert!(!second.context_is_current(&foreign));
    assert_eq!(
        second.clear_from_context(&foreign, SettingsScope::Project, &snap_key),
        Err(SettingsMutationError::ForeignMutationContext)
    );
    // Session edits must also reject another project's captured context before changing values.
    assert_eq!(
        second.set_from_context(
            &foreign,
            SettingsScope::Session,
            &snap_key,
            SettingValue::Float(2.5),
        ),
        Err(SettingsMutationError::ForeignMutationContext)
    );
    assert_eq!(
        second.clear_from_context(&local, SettingsScope::Project, &snap_key),
        Ok(None)
    );
    assert_eq!(
        second.authority().snapshot().generation(),
        local.generation()
    );
    assert_eq!(
        second.capture_values([&snap_key]).unwrap().values(),
        local.values()
    );

    // A retired coordinator's context cannot alias a replacement with the same local counters.
    first.flush_then_shutdown().unwrap().finish().unwrap();
    drop(first);
    let replacement = SettingsMutationCoordinator::in_memory_with_defaults();
    replacement.bind_project(first_project.root()).unwrap();
    assert!(!replacement.context_is_current(&foreign));
    assert_eq!(
        replacement.clear_from_context(&foreign, SettingsScope::Project, &snap_key),
        Err(SettingsMutationError::ForeignMutationContext)
    );
    replacement.flush_then_shutdown().unwrap().finish().unwrap();
    second.flush_then_shutdown().unwrap().finish().unwrap();
    assert!(!first_project.root().join(".zircon").exists());
    assert!(!second_project.root().join(".zircon").exists());
}

fn guarded_retry_coordinator(root: &Path) -> SettingsMutationCoordinator {
    let authority = Arc::new(SettingsAuthority::with_defaults());
    let persistence = SettingsPersistenceService::with_limits(
        Arc::clone(&authority),
        crate::core::jobs::test_job_scheduler(),
        SettingsPersistenceLimits {
            max_entries: 1,
            max_retained_bytes: 1,
        },
    );
    SettingsMutationCoordinator::new(
        authority,
        persistence,
        Some(SettingsStore::from_roots(root, None)),
    )
}

#[test]
fn guarded_preferences_retry_rejects_replaced_pending_document_and_preserves_current_change() {
    let fixture =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_retry_replace").unwrap();
    let coordinator = guarded_retry_coordinator(fixture.root());
    let setting = key(crate::core::settings::EDITOR_SHOW_PREVIEW_FEATURES_KEY);
    let first = coordinator
        .set(SettingsScope::User, &setting, SettingValue::Bool(true))
        .unwrap();
    let (health, contexts) = coordinator.capture_persistence_health_with_retry().unwrap();
    let displayed = contexts
        .into_iter()
        .find(|context| context.scope() == SettingsScope::User)
        .unwrap();
    assert_eq!(
        Some(displayed.file_generation()),
        first.persistence_generation()
    );
    assert_eq!(
        health.user().file_generation(),
        first.persistence_generation()
    );
    let retry = coordinator
        .retry_pending_from_context(&displayed, SettingsScope::User)
        .unwrap();
    assert_eq!(retry.file_generation(), first.persistence_generation());
    assert!(matches!(
        retry.disposition(),
        SettingsPersistenceRetryDisposition::PendingAdmission(_)
    ));

    let replacement = coordinator
        .set(SettingsScope::User, &setting, SettingValue::Bool(false))
        .unwrap();
    assert_ne!(
        replacement.persistence_generation(),
        first.persistence_generation()
    );
    let before_health = coordinator.persistence_health_snapshot();
    let before_generation = coordinator.authority().snapshot().generation();
    assert_eq!(
        coordinator.retry_pending_from_context(&displayed, SettingsScope::User),
        Err(SettingsMutationError::StalePersistenceRetryContext)
    );
    assert_eq!(coordinator.persistence_health_snapshot(), before_health);
    assert_eq!(
        coordinator.authority().snapshot().generation(),
        before_generation
    );
    assert_eq!(
        coordinator
            .authority()
            .resolved_setting(&setting)
            .unwrap()
            .value(),
        &SettingValue::Bool(false)
    );

    // A rejected target releases the original operation phase and retains the new document.
    let (_, current) = coordinator.capture_persistence_health_with_retry().unwrap();
    let current = current
        .into_iter()
        .find(|context| context.scope() == SettingsScope::User)
        .unwrap();
    assert_eq!(
        Some(current.file_generation()),
        replacement.persistence_generation()
    );
    let retry = coordinator
        .retry_pending_from_context(&current, SettingsScope::User)
        .unwrap();
    assert_eq!(
        retry.file_generation(),
        replacement.persistence_generation()
    );
    assert!(matches!(
        retry.disposition(),
        SettingsPersistenceRetryDisposition::PendingAdmission(_)
    ));
    assert_eq!(
        coordinator
            .retry_pending(SettingsScope::User)
            .unwrap()
            .file_generation(),
        replacement.persistence_generation()
    );
    coordinator.shutdown().wait();
    assert!(!fixture.root().join("settings.toml").exists());
}

#[test]
fn guarded_preferences_retry_rejects_project_rebinding_and_wrong_scope_without_retiring_pending() {
    let first_root =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_retry_project_first").unwrap();
    let second_root =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_retry_project_second")
            .unwrap();
    let coordinator = guarded_retry_coordinator(first_root.root());
    let setting = key(VIEWPORT_TRANSLATE_STEP_KEY);
    coordinator.bind_project(first_root.root()).unwrap();
    coordinator
        .set(SettingsScope::Project, &setting, SettingValue::Float(2.5))
        .unwrap();
    let (_, contexts) = coordinator.capture_persistence_health_with_retry().unwrap();
    let displayed = contexts
        .into_iter()
        .find(|context| context.scope() == SettingsScope::Project)
        .unwrap();
    let before = coordinator.persistence_health_snapshot();
    assert_eq!(
        coordinator.retry_pending_from_context(&displayed, SettingsScope::User),
        Err(SettingsMutationError::StalePersistenceRetryContext)
    );
    assert_eq!(coordinator.persistence_health_snapshot(), before);

    coordinator.clear_project().unwrap();
    coordinator.bind_project(second_root.root()).unwrap();
    let replacement = coordinator
        .set(SettingsScope::Project, &setting, SettingValue::Float(3.5))
        .unwrap();
    let before = coordinator.persistence_health_snapshot();
    let value_generation = coordinator.authority().snapshot().generation();
    assert_eq!(
        coordinator.retry_pending_from_context(&displayed, SettingsScope::Project),
        Err(SettingsMutationError::StaleProjectBinding)
    );
    assert_eq!(coordinator.persistence_health_snapshot(), before);
    assert_eq!(
        coordinator.authority().snapshot().generation(),
        value_generation
    );
    let (_, current) = coordinator.capture_persistence_health_with_retry().unwrap();
    let current = current
        .into_iter()
        .find(|context| context.scope() == SettingsScope::Project)
        .unwrap();
    assert_eq!(
        coordinator
            .retry_pending_from_context(&current, SettingsScope::Project)
            .unwrap()
            .file_generation(),
        replacement.persistence_generation()
    );
    coordinator.shutdown().wait();
}

#[test]
fn guarded_preferences_retry_rejects_foreign_and_released_coordinator_contexts() {
    let first_root =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_retry_owner_first").unwrap();
    let second_root =
        crate::tests::support::OwnedSourceWriteFixture::new("guarded_retry_owner_second").unwrap();
    let first = guarded_retry_coordinator(first_root.root());
    let second = guarded_retry_coordinator(second_root.root());
    let setting = key(crate::core::settings::EDITOR_SHOW_PREVIEW_FEATURES_KEY);
    first
        .set(SettingsScope::User, &setting, SettingValue::Bool(true))
        .unwrap();
    let pending = second
        .set(SettingsScope::User, &setting, SettingValue::Bool(true))
        .unwrap();
    let (_, contexts) = first.capture_persistence_health_with_retry().unwrap();
    let displayed = contexts
        .into_iter()
        .find(|context| context.scope() == SettingsScope::User)
        .unwrap();
    let before = second.persistence_health_snapshot();
    assert_eq!(
        second.retry_pending_from_context(&displayed, SettingsScope::User),
        Err(SettingsMutationError::ForeignMutationContext)
    );
    assert_eq!(second.persistence_health_snapshot(), before);
    first.shutdown().wait();
    drop(first);
    assert_eq!(
        second.retry_pending_from_context(&displayed, SettingsScope::User),
        Err(SettingsMutationError::ForeignMutationContext)
    );
    assert_eq!(second.persistence_health_snapshot(), before);
    let (_, current) = second.capture_persistence_health_with_retry().unwrap();
    let current = current
        .into_iter()
        .find(|context| context.scope() == SettingsScope::User)
        .unwrap();
    assert_eq!(
        second
            .retry_pending_from_context(&current, SettingsScope::User)
            .unwrap()
            .file_generation(),
        pending.persistence_generation()
    );
    second.shutdown().wait();
}

#[test]
fn popup_context_check_rejects_foreign_and_stale_values_without_mutating_the_transaction() {
    let coordinator = SettingsMutationCoordinator::in_memory_with_defaults();
    let foreign = SettingsMutationCoordinator::in_memory_with_defaults();
    let locale = key(crate::core::settings::EDITOR_LOCALE_KEY);
    let displayed = coordinator.capture_values([&locale]).unwrap();
    let before_health = coordinator.persistence_health_snapshot();
    assert_eq!(coordinator.check_values_context(&displayed), Ok(()));
    assert_eq!(
        foreign.check_values_context(&displayed),
        Err(SettingsMutationError::ForeignMutationContext)
    );
    assert_eq!(coordinator.check_values_context(&displayed), Ok(()));
    coordinator
        .authority()
        .set(
            SettingsScope::Session,
            &locale,
            SettingValue::Enum("zh-CN".to_owned()),
        )
        .unwrap();
    let generation = coordinator.authority().snapshot().generation();
    assert_eq!(
        coordinator.check_values_context(&displayed),
        Err(SettingsMutationError::Settings(
            SettingsError::StaleGeneration {
                expected: displayed.generation(),
                actual: generation,
            }
        ))
    );
    assert_eq!(coordinator.persistence_health_snapshot(), before_health);
    assert_eq!(coordinator.authority().snapshot().generation(), generation);
    assert_eq!(
        coordinator
            .authority()
            .resolved_setting(&locale)
            .unwrap()
            .value(),
        &SettingValue::Enum("zh-CN".to_owned())
    );
    let current = coordinator.capture_values([&locale]).unwrap();
    assert_eq!(coordinator.check_values_context(&current), Ok(()));
    assert!(coordinator.context_is_current(&current));
    coordinator.shutdown().wait();
    foreign.shutdown().wait();
}

#[test]
fn popup_context_check_rejects_project_rebinding_and_releases_no_new_phase() {
    let first = crate::tests::support::OwnedSourceWriteFixture::new("popup_binding_first").unwrap();
    let second =
        crate::tests::support::OwnedSourceWriteFixture::new("popup_binding_second").unwrap();
    let coordinator = SettingsMutationCoordinator::in_memory_with_defaults();
    let locale = key(crate::core::settings::EDITOR_LOCALE_KEY);
    coordinator.bind_project(first.root()).unwrap();
    let displayed = coordinator.capture_values([&locale]).unwrap();
    coordinator.clear_project().unwrap();
    coordinator.bind_project(second.root()).unwrap();
    let before_health = coordinator.persistence_health_snapshot();
    let before_generation = coordinator.authority().snapshot().generation();
    assert_eq!(
        coordinator.check_values_context(&displayed),
        Err(SettingsMutationError::StaleProjectBinding)
    );
    assert_eq!(coordinator.persistence_health_snapshot(), before_health);
    assert_eq!(
        coordinator.authority().snapshot().generation(),
        before_generation
    );
    let current = coordinator.capture_values([&locale]).unwrap();
    assert_eq!(coordinator.check_values_context(&current), Ok(()));
    assert!(coordinator
        .set_from_context(
            &current,
            SettingsScope::Session,
            &locale,
            SettingValue::Enum("zh-CN".to_owned()),
        )
        .unwrap()
        .changed());
    coordinator.shutdown().wait();
}

#[test]
fn projection_values_capture_rejects_stale_values_without_pinning_the_phase() {
    let coordinator = SettingsMutationCoordinator::in_memory_with_defaults();
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let expected = coordinator.authority().snapshot().generation();
    coordinator
        .authority()
        .set(SettingsScope::Session, &snap_key, SettingValue::Float(2.5))
        .unwrap();
    let actual = coordinator.authority().snapshot().generation();
    let health = coordinator.persistence_health_snapshot();
    assert_eq!(
        coordinator.capture_values_at_generation(expected, [&snap_key]),
        Err(SettingsMutationError::Settings(
            SettingsError::StaleGeneration { expected, actual }
        ))
    );
    assert_eq!(coordinator.persistence_health_snapshot(), health);
    assert_eq!(coordinator.authority().snapshot().generation(), actual);
    assert_eq!(
        coordinator
            .authority()
            .resolved_setting(&snap_key)
            .unwrap()
            .value(),
        &SettingValue::Float(2.5)
    );
    let current = coordinator
        .capture_values_at_generation(actual, [&snap_key])
        .unwrap();
    assert_eq!(current.generation(), actual);
    assert_eq!(
        current.value(&snap_key).unwrap().value(),
        &SettingValue::Float(2.5)
    );
    assert_eq!(
        coordinator
            .set_from_context(
                &current,
                SettingsScope::Session,
                &snap_key,
                SettingValue::Float(5.0)
            )
            .unwrap()
            .disposition(),
        SettingsMutationDisposition::SessionApplied
    );
    coordinator.shutdown().wait();
}

#[test]
fn projection_values_capture_keeps_project_identity_at_equal_value_generations() {
    let first =
        crate::tests::support::OwnedSourceWriteFixture::new("projection_values_first").unwrap();
    let second =
        crate::tests::support::OwnedSourceWriteFixture::new("projection_values_second").unwrap();
    let coordinator = SettingsMutationCoordinator::in_memory_with_defaults();
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    coordinator.bind_project(first.root()).unwrap();
    let expected = coordinator.authority().snapshot().generation();
    let first_context = coordinator
        .capture_values_at_generation(expected, [&snap_key])
        .unwrap();
    coordinator.clear_project().unwrap();
    coordinator.bind_project(second.root()).unwrap();
    assert_eq!(coordinator.authority().snapshot().generation(), expected);
    assert_eq!(
        coordinator.set_from_context(
            &first_context,
            SettingsScope::Session,
            &snap_key,
            SettingValue::Float(2.5)
        ),
        Err(SettingsMutationError::StaleProjectBinding)
    );
    let current = coordinator
        .capture_values_at_generation(expected, [&snap_key])
        .unwrap();
    assert_eq!(current.generation(), first_context.generation());
    assert!(coordinator.context_is_current(&current));
    assert!(!coordinator.context_is_current(&first_context));
    assert_eq!(
        coordinator
            .set_from_context(
                &current,
                SettingsScope::Session,
                &snap_key,
                SettingValue::Float(2.5)
            )
            .unwrap()
            .disposition(),
        SettingsMutationDisposition::SessionApplied
    );
    coordinator.shutdown().wait();
}

#[test]
fn projection_values_capture_keeps_exact_u64_and_rejects_exhaustion() {
    for generation in [i64::MAX as u64 + 9, u64::MAX] {
        let mut registry = settings_registry_with_defaults();
        registry.revision = generation;
        let authority = Arc::new(SettingsAuthority::from_registry_for_performance_benchmark(
            registry,
        ));
        let persistence = SettingsPersistenceService::new(
            Arc::clone(&authority),
            crate::core::jobs::test_job_scheduler(),
        );
        let coordinator =
            SettingsMutationCoordinator::new(Arc::clone(&authority), persistence, None);
        let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
        assert_eq!(
            coordinator.capture_values_at_generation(i64::MAX as u64, [&snap_key]),
            Err(SettingsMutationError::Settings(
                SettingsError::StaleGeneration {
                    expected: i64::MAX as u64,
                    actual: generation,
                }
            ))
        );
        if generation == u64::MAX {
            assert_eq!(
                coordinator.capture_values_at_generation(generation, [&snap_key]),
                Err(SettingsMutationError::Settings(
                    SettingsError::GenerationExhausted
                ))
            );
        } else {
            let captured = coordinator
                .capture_values_at_generation(generation, [&snap_key])
                .unwrap();
            assert_eq!(captured.generation(), generation);
            assert!(captured.value(&snap_key).is_some());
            assert!(coordinator.context_is_current(&captured));
        }
        assert_eq!(authority.snapshot().generation(), generation);
        coordinator.shutdown().wait();
    }
}
