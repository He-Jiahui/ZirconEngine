use super::*;

#[test]
fn persistence_service_writes_the_authority_layer_from_a_typed_change_ticket() {
    let root = temporary_root("persistence-ticket");
    let project_root = root.join("project");
    let store = SettingsStore::from_roots(&root, Some(&project_root));
    let authority = std::sync::Arc::new(SettingsAuthority::with_defaults());
    assert!(matches!(
        authority.load_project_layer_from_store(&store),
        SettingsProjectLayerLoad::Missing { .. }
    ));
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    authority
        .set(SettingsScope::Session, &snap_key, SettingValue::Float(1.5))
        .unwrap();
    let change = authority
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(2.5))
        .unwrap()
        .expect("a changed setting must publish a persistence request");
    let service = SettingsPersistenceService::new(
        std::sync::Arc::clone(&authority),
        crate::core::jobs::test_job_scheduler(),
    );

    let ticket = service.submit(&change, store.clone()).unwrap();
    assert_eq!(ticket.key(), &snap_key);
    assert_eq!(ticket.scope(), SettingsScope::Project);
    assert!(ticket.file_generation().get() > 0);
    assert_eq!(ticket.authority_generation(), change.revision);
    assert!(ticket.target().starts_with("settings:project:"));
    assert!(matches!(
        ticket.wait_until(Instant::now() + Duration::from_secs(5)),
        zircon_runtime::core::runtime::tasks::BoundedKeyedIoWaitResult::Terminal(
            zircon_runtime::core::runtime::tasks::BoundedKeyedIoTerminal::Succeeded
        )
    ));

    let mut restored = settings_registry_with_defaults();
    assert!(matches!(
        store.load_into(SettingsScope::Project, &mut restored),
        Ok(SettingsLoad::Loaded { .. })
    ));
    assert_eq!(
        restored.resolve(&snap_key).unwrap(),
        &SettingValue::Float(2.5)
    );
    remove_temporary_root(&root);
}

#[test]
fn project_save_never_serializes_a_replaced_project_authority_layer() {
    let root = temporary_root("project-save-binding");
    let project_a = root.join("project-a");
    let project_b = root.join("project-b");
    let store_a = SettingsStore::from_roots(root.join("user"), Some(&project_a));
    let store_b = SettingsStore::from_roots(root.join("user"), Some(&project_b));
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);

    let mut source_a = settings_registry_with_defaults();
    source_a
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(1.5))
        .unwrap();
    store_a
        .save_from(SettingsScope::Project, &source_a)
        .unwrap();
    let mut source_b = settings_registry_with_defaults();
    source_b
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(3.5))
        .unwrap();
    store_b
        .save_from(SettingsScope::Project, &source_b)
        .unwrap();

    let authority = SettingsAuthority::with_defaults();
    assert!(matches!(
        authority.load_project_layer_from_store(&store_a),
        SettingsProjectLayerLoad::Persisted { .. }
    ));
    authority
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(2.5))
        .unwrap();
    authority.clear_project_layer();
    assert!(matches!(
        authority.load_project_layer_from_store(&store_b),
        SettingsProjectLayerLoad::Persisted { .. }
    ));

    store_a
        .save_authority_layer(SettingsScope::Project, &authority)
        .unwrap();
    let mut restored_a = settings_registry_with_defaults();
    store_a
        .load_into(SettingsScope::Project, &mut restored_a)
        .unwrap();
    assert_eq!(
        restored_a.resolve(&snap_key).unwrap(),
        &SettingValue::Float(1.5),
        "a stale Project A worker must not write Project B values to Project A"
    );

    remove_temporary_root(&root);
}

#[test]
fn persistence_service_retries_a_failed_typed_request_with_a_new_ticket() {
    let root = temporary_root("persistence-retry");
    fs::write(&root, "a file blocks the settings directory").unwrap();
    let store = SettingsStore::from_roots(&root, None);
    let authority = std::sync::Arc::new(SettingsAuthority::with_defaults());
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let change = authority
        .set(SettingsScope::User, &snap_key, SettingValue::Float(2.5))
        .unwrap()
        .expect("a changed setting must publish a persistence request");
    let service =
        SettingsPersistenceService::new(authority, crate::core::jobs::test_job_scheduler());

    let failed = service.submit(&change, store.clone()).unwrap();
    assert!(matches!(
        failed.wait_until(Instant::now() + Duration::from_secs(5)),
        zircon_runtime::core::runtime::tasks::BoundedKeyedIoWaitResult::Terminal(
            zircon_runtime::core::runtime::tasks::BoundedKeyedIoTerminal::Failed(_)
        )
    ));

    fs::remove_file(&root).unwrap();
    fs::create_dir_all(&root).unwrap();
    let retried = service.retry(&failed).unwrap();
    assert_eq!(retried.key(), failed.key());
    assert_eq!(retried.scope(), failed.scope());
    assert_eq!(retried.target(), failed.target());
    assert_eq!(retried.file_generation(), failed.file_generation());
    assert_eq!(
        retried.authority_generation(),
        failed.authority_generation()
    );
    assert!(matches!(
        retried.wait_until(Instant::now() + Duration::from_secs(5)),
        zircon_runtime::core::runtime::tasks::BoundedKeyedIoWaitResult::Terminal(
            zircon_runtime::core::runtime::tasks::BoundedKeyedIoTerminal::Succeeded
        )
    ));

    remove_temporary_root(&root);
}

#[test]
fn persistence_service_fences_admitted_writes_before_shutdown() {
    let root = temporary_root("persistence-shutdown");
    let project_root = root.join("project");
    let store = SettingsStore::from_roots(&root, Some(&project_root));
    let authority = std::sync::Arc::new(SettingsAuthority::with_defaults());
    assert!(matches!(
        authority.load_project_layer_from_store(&store),
        SettingsProjectLayerLoad::Missing { .. }
    ));
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let change = authority
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(2.5))
        .unwrap()
        .expect("a changed setting must publish a persistence request");
    let service =
        SettingsPersistenceService::new(authority, crate::core::jobs::test_job_scheduler());

    service.submit(&change, store.clone()).unwrap();
    let report = service.flush_then_shutdown().unwrap().finish().unwrap();
    assert_eq!(report.incomplete_entries, 0);

    let mut restored = settings_registry_with_defaults();
    assert!(matches!(
        store.load_into(SettingsScope::Project, &mut restored),
        Ok(SettingsLoad::Loaded { .. })
    ));
    assert_eq!(
        restored.resolve(&snap_key).unwrap(),
        &SettingValue::Float(2.5)
    );
    remove_temporary_root(&root);
}

#[test]
fn persistence_service_shutdown_reports_a_failed_fenced_write() {
    let root = temporary_root("persistence-shutdown-failure");
    fs::write(&root, "a file blocks the settings directory").unwrap();
    let store = SettingsStore::from_roots(&root, None);
    let authority = std::sync::Arc::new(SettingsAuthority::with_defaults());
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let change = authority
        .set(SettingsScope::User, &snap_key, SettingValue::Float(2.5))
        .unwrap()
        .expect("a changed setting must publish a persistence request");
    let service =
        SettingsPersistenceService::new(authority, crate::core::jobs::test_job_scheduler());

    service.submit(&change, store).unwrap();
    assert!(matches!(
        service.flush_then_shutdown().unwrap().finish(),
        Err(SettingsPersistenceShutdownError::FenceTerminal(
            zircon_runtime::core::runtime::tasks::BoundedKeyedIoTerminal::Failed(_)
        ))
    ));

    fs::remove_file(&root).unwrap();
}

#[test]
fn persistence_service_rejects_session_only_changes_before_lane_admission() {
    let authority = std::sync::Arc::new(SettingsAuthority::with_defaults());
    let mru_key = key(EDITOR_COMMAND_PALETTE_MRU_KEY);
    let change = authority
        .set(
            SettingsScope::Session,
            &mru_key,
            SettingValue::CommandPaletteMru(EditorCommandPaletteMru::default()),
        )
        .unwrap();
    assert!(
        change.is_none(),
        "an unchanged session value must not enqueue work"
    );

    let service =
        SettingsPersistenceService::new(authority, crate::core::jobs::test_job_scheduler());
    let change = crate::core::settings::SettingChange {
        key: mru_key,
        scope: SettingsScope::Session,
        revision: 1,
        requires_restart: false,
    };
    let store = SettingsStore::from_roots(temporary_root("session-ticket"), None);

    assert!(matches!(
        service.submit(&change, store),
        Err(super::SettingsPersistenceSubmitError::NonPersistentScope(
            SettingsScope::Session
        ))
    ));
}

#[test]
fn persistence_service_rejects_a_request_before_retaining_over_its_byte_budget() {
    let authority = std::sync::Arc::new(SettingsAuthority::with_defaults());
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let change = authority
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(4.0))
        .unwrap()
        .unwrap();
    let service = SettingsPersistenceService::with_limits(
        authority,
        crate::core::jobs::test_job_scheduler(),
        SettingsPersistenceLimits {
            max_entries: 1,
            max_retained_bytes: 1,
        },
    );
    let root = temporary_root("byte-budget");
    let store = SettingsStore::from_roots(&root, Some(&root));

    assert!(matches!(
        service.submit(&change, store),
        Err(super::SettingsPersistenceSubmitError::LaneAdmission(
            zircon_runtime::core::runtime::tasks::BoundedKeyedIoAdmissionError::RetainedBytesCapacityExceeded
        ))
    ));
}
