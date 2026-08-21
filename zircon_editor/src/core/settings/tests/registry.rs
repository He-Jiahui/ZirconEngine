use super::*;

#[test]
fn resolve_uses_session_project_user_and_default_precedence() {
    let definition = project_grid_setting();
    let key = definition.key.clone();
    let mut registry = SettingsRegistry::default();
    registry.register(definition).unwrap();

    assert_eq!(registry.resolve(&key).unwrap(), &SettingValue::Int(10));
    registry
        .set(SettingsScope::User, &key, SettingValue::Int(12))
        .unwrap();
    registry
        .set(SettingsScope::Project, &key, SettingValue::Int(16))
        .unwrap();
    registry
        .set(SettingsScope::Session, &key, SettingValue::Int(20))
        .unwrap();
    assert_eq!(registry.resolve(&key).unwrap(), &SettingValue::Int(20));

    registry.clear(SettingsScope::Session, &key).unwrap();
    assert_eq!(registry.resolve(&key).unwrap(), &SettingValue::Int(16));
    registry.clear(SettingsScope::Project, &key).unwrap();
    assert_eq!(registry.resolve(&key).unwrap(), &SettingValue::Int(12));
}

#[test]
fn built_in_setting_presentations_use_direct_embedded_localization_keys() {
    let registry = settings_registry_with_defaults();
    let catalog = crate::core::i18n::EditorI18nCatalog::embedded().unwrap();
    let settings = [
        EDITOR_DESIGN_TOKENS_KEY,
        EDITOR_KEYMAP_OVERRIDES_KEY,
        EDITOR_COMMAND_PALETTE_MRU_KEY,
        EDITOR_LOCALE_KEY,
        VIEWPORT_TRANSLATE_STEP_KEY,
        VIEWPORT_ROTATE_STEP_DEGREES_KEY,
        VIEWPORT_SCALE_STEP_KEY,
    ];

    for setting in settings {
        let presentation = registry.definition(&key(setting)).unwrap().presentation();
        assert!(presentation.label_key().starts_with("settings."));
        assert!(presentation.description_key().starts_with("settings."));
        assert!(presentation
            .category_path()
            .all(|key| key.starts_with("settings.category.")));
        for locale in catalog.available_locales() {
            assert!(
                embedded_bundle_contains_translation(&locale, presentation.label_key()),
                "{setting} has no direct label translation in {}",
                locale.as_str()
            );
            assert!(
                embedded_bundle_contains_translation(&locale, presentation.description_key()),
                "{setting} has no direct description translation in {}",
                locale.as_str()
            );
            assert_ne!(
                catalog
                    .translate_for_locale(&locale, presentation.label_key())
                    .as_ref(),
                presentation.label_key()
            );
            assert_ne!(
                catalog
                    .translate_for_locale(&locale, presentation.description_key())
                    .as_ref(),
                presentation.description_key()
            );
            for category in presentation.category_path() {
                assert!(
                    embedded_bundle_contains_translation(&locale, category),
                    "{setting} has no direct category translation in {}",
                    locale.as_str()
                );
                assert_ne!(
                    catalog.translate_for_locale(&locale, category).as_ref(),
                    category,
                    "{setting} has an unresolved category in {}",
                    locale.as_str()
                );
            }
        }
    }
}

#[test]
fn setting_presentation_rejects_literal_and_empty_category_metadata() {
    assert!(SettingsPresentation::new(
        "Appearance",
        "settings.editor.appearance.description",
        ["settings.category.appearance"],
    )
    .is_err());
    assert!(SettingsPresentation::new(
        "settings.editor.appearance.label",
        "settings.editor.appearance.description",
        std::iter::empty::<&str>(),
    )
    .is_err());
    assert!(SettingsPresentation::new(
        "settings.editor.",
        "settings.editor.appearance.description",
        ["settings.category.appearance"],
    )
    .is_err());
    assert!(SettingsPresentation::new(
        "settings.editor.appearance.label",
        "settings.editor.appearance.description",
        ["settings.category."],
    )
    .is_err());
}

#[test]
fn authority_notifies_reentrant_change_subscribers_after_releasing_state() {
    let authority = Arc::new(SettingsAuthority::with_defaults());
    let (observed, receiver) = mpsc::channel();
    authority.configure_change_subscriber(Arc::new(ReentrantSettingsChangeSubscriber {
        authority: Arc::clone(&authority),
        observed,
    }));
    let viewport_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let writer = Arc::clone(&authority);

    let mutation = thread::spawn(move || {
        writer.set(
            SettingsScope::Project,
            &viewport_key,
            SettingValue::Float(2.0),
        )
    });

    assert_eq!(
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("change subscriber should read the bounded delta outside the state lock"),
        (1, 1, 1)
    );
    assert!(mutation.join().unwrap().unwrap().is_some());
}

#[test]
fn project_layer_transition_rejects_reentrant_persistence_prepare() {
    let root = temporary_root("project-layer-reentrant-prepare");
    let project_root = root.join("project");
    let store = SettingsStore::from_roots(root.join("user"), Some(&project_root));
    let snap_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let mut source = settings_registry_with_defaults();
    source
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(2.5))
        .unwrap();
    store.save_from(SettingsScope::Project, &source).unwrap();

    let authority = Arc::new(SettingsAuthority::with_defaults());
    let (observed, receiver) = mpsc::channel();
    authority.configure_change_subscriber(Arc::new(ProjectLayerPrepareSubscriber {
        authority: Arc::clone(&authority),
        store: store.clone(),
        observed,
    }));

    let load_authority = Arc::clone(&authority);
    let load_store = store.clone();
    let load = thread::spawn(move || load_authority.load_project_layer_from_store(&load_store));
    assert!(receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("project-layer load callback must not deadlock"));
    assert!(matches!(
        load.join().expect("project-layer load should not panic"),
        SettingsProjectLayerLoad::Persisted { .. }
    ));

    let clear_authority = Arc::clone(&authority);
    let clear = thread::spawn(move || clear_authority.clear_project_layer());
    assert!(receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("project-layer clear callback must not deadlock"));
    clear.join().expect("project-layer clear should not panic");
    remove_temporary_root(&root);
}

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
    let change = authority
        .set(SettingsScope::Project, &snap_key, SettingValue::Float(2.5))
        .unwrap()
        .expect("a changed setting must publish a persistence request");
    let service = SettingsPersistenceService::new(std::sync::Arc::clone(&authority));

    let ticket = service.submit(&change, store.clone()).unwrap();
    assert_eq!(ticket.key(), &snap_key);
    assert_eq!(ticket.scope(), SettingsScope::Project);
    assert_eq!(ticket.generation(), change.revision);
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
    let service = SettingsPersistenceService::new(authority);

    let failed = service.submit(&change, store.clone()).unwrap();
    assert!(matches!(
        failed.wait_until(Instant::now() + Duration::from_secs(5)),
        zircon_runtime::core::runtime::tasks::BoundedKeyedIoWaitResult::Terminal(
            zircon_runtime::core::runtime::tasks::BoundedKeyedIoTerminal::Failed(_)
        )
    ));

    fs::remove_file(&root).unwrap();
    fs::create_dir_all(&root).unwrap();
    let retried = service.retry(&failed, store.clone()).unwrap();
    assert_eq!(retried.key(), failed.key());
    assert_eq!(retried.scope(), failed.scope());
    assert_eq!(retried.generation(), failed.generation());
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
    let service = SettingsPersistenceService::new(authority);

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
    let service = SettingsPersistenceService::new(authority);

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

    let service = SettingsPersistenceService::new(authority);
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
        SettingsPersistenceLimits {
            max_entries: 1,
            max_retained_bytes: 1,
        },
    );
    let store = SettingsStore::from_roots(temporary_root("byte-budget"), None);

    assert!(matches!(
        service.submit(&change, store),
        Err(super::SettingsPersistenceSubmitError::LaneAdmission(
            zircon_runtime::core::runtime::tasks::BoundedKeyedIoAdmissionError::RetainedBytesCapacityExceeded
        ))
    ));
}

#[test]
fn authority_publishes_command_palette_mru_through_the_session_layer() {
    let authority = SettingsAuthority::with_defaults();
    let mru_key = key(EDITOR_COMMAND_PALETTE_MRU_KEY);
    let command = EditorOperationPath::parse("editor.command_palette.open").unwrap();
    let mut expected = authority.snapshot().command_palette_mru().clone();
    assert!(expected.record(command));

    let change = authority
        .set(
            SettingsScope::Session,
            &mru_key,
            SettingValue::CommandPaletteMru(expected.clone()),
        )
        .unwrap()
        .expect("a changed session MRU must publish a new authority generation");

    assert_eq!(change.scope, SettingsScope::Session);
    assert_eq!(authority.snapshot().command_palette_mru(), &expected);
}

#[test]
fn authority_records_palette_usage_without_a_leaf_key_or_duplicate_mru_value() {
    let authority = SettingsAuthority::with_defaults();
    let before = authority.snapshot();
    let command = EditorOperationPath::parse("editor.command_palette.open").unwrap();

    let change = authority
        .record_command_palette_usage(command.clone())
        .unwrap()
        .expect("the first command usage must update the bounded MRU setting");
    let after = authority.snapshot();

    assert_eq!(change.scope, SettingsScope::Session);
    assert_eq!(
        after.command_palette_mru().entries(),
        std::slice::from_ref(&command)
    );
    assert!(std::ptr::eq(before.design_tokens(), after.design_tokens()));
    assert!(std::ptr::eq(
        before.keymap_overrides(),
        after.keymap_overrides()
    ));
    assert!(authority
        .record_command_palette_usage(
            EditorOperationPath::parse("editor.command_palette.open").unwrap()
        )
        .unwrap()
        .is_none());
    assert!(std::sync::Arc::ptr_eq(&after, &authority.snapshot()));
}

#[test]
fn schema_scope_and_change_contracts_fail_closed() {
    let key = key("editor.autosave.interval_secs");
    let definition = SettingDefinition::new(
        key.clone(),
        SettingsScope::User,
        SettingSchema::Enum {
            variants: BTreeSet::from(["60".to_string(), "300".to_string()]),
        },
        SettingValue::Enum("300".to_string()),
        true,
        presentation(
            "settings.editor.autosave.interval_secs.label",
            "settings.editor.autosave.interval_secs.description",
            &["settings.category.editor", "settings.category.autosave"],
        ),
    )
    .unwrap();
    let mut registry = SettingsRegistry::default();
    registry.register(definition).unwrap();

    assert!(matches!(
        registry.set(
            SettingsScope::Project,
            &key,
            SettingValue::Enum("60".to_string())
        ),
        Err(SettingsError::ScopeNotAllowed { .. })
    ));
    assert!(matches!(
        registry.set(
            SettingsScope::User,
            &key,
            SettingValue::Enum("15".to_string())
        ),
        Err(SettingsError::InvalidValue { .. })
    ));

    let change = registry
        .set(
            SettingsScope::Session,
            &key,
            SettingValue::Enum("60".to_string()),
        )
        .unwrap()
        .expect("a changed scoped value should emit a change");
    assert!(change.requires_restart);
    assert_eq!(change.revision, 1);
    let delta = registry.changes_since(SettingsChangeCursor::origin());
    assert_eq!(delta.changes, vec![change]);
    assert!(!delta.requires_snapshot);
    assert_eq!(delta.cursor.revision(), 1);
}

#[test]
fn unchanged_setting_value_does_not_advance_revision_or_emit_a_delta() {
    let definition = project_grid_setting();
    let key = definition.key.clone();
    let mut registry = SettingsRegistry::default();
    registry.register(definition).unwrap();

    let changed = registry
        .set(SettingsScope::Project, &key, SettingValue::Int(16))
        .unwrap()
        .expect("a changed scoped value should emit a change");
    let cursor = registry.change_cursor();
    let repeated = registry
        .set(SettingsScope::Project, &key, SettingValue::Int(16))
        .unwrap();

    assert!(repeated.is_none());
    assert_eq!(changed.revision, cursor.revision());
    assert_eq!(registry.change_cursor(), cursor);
    assert!(registry.changes_since(cursor).changes.is_empty());
}

#[test]
fn retained_change_delta_requires_a_snapshot_after_cursor_falls_behind_entry_budget() {
    let definition = project_grid_setting();
    let key = definition.key.clone();
    let mut registry = SettingsRegistry::with_change_log_policy(SettingsChangeLogPolicy::new(
        2,
        usize::MAX,
        Duration::from_secs(60),
    ));
    registry.register(definition).unwrap();

    registry
        .set(SettingsScope::Project, &key, SettingValue::Int(11))
        .unwrap();
    let second = registry
        .set(SettingsScope::Project, &key, SettingValue::Int(12))
        .unwrap();
    let third = registry
        .set(SettingsScope::Project, &key, SettingValue::Int(13))
        .unwrap();

    let delta = registry.changes_since(SettingsChangeCursor::origin());
    assert!(delta.requires_snapshot);
    assert_eq!(delta.changes, vec![second.unwrap(), third.unwrap()]);
    assert_eq!(delta.cursor.revision(), 3);
}

#[test]
fn retained_change_delta_requires_a_snapshot_after_byte_budget_evicts_the_log() {
    let definition = project_grid_setting();
    let key = definition.key.clone();
    let mut registry = SettingsRegistry::with_change_log_policy(SettingsChangeLogPolicy::new(
        usize::MAX,
        1,
        Duration::from_secs(60),
    ));
    registry.register(definition).unwrap();

    registry
        .set(SettingsScope::Project, &key, SettingValue::Int(11))
        .unwrap()
        .expect("a changed value should enter the bounded change log");

    let delta = registry.changes_since(SettingsChangeCursor::origin());

    assert!(delta.requires_snapshot);
    assert!(delta.changes.is_empty());
    assert_eq!(delta.cursor.revision(), 1);
}

#[test]
fn retained_change_delta_requires_a_snapshot_after_age_budget_evicts_the_log() {
    let definition = project_grid_setting();
    let key = definition.key.clone();
    let mut registry = SettingsRegistry::with_change_log_policy(SettingsChangeLogPolicy::new(
        8,
        usize::MAX,
        Duration::ZERO,
    ));
    registry.register(definition).unwrap();

    registry
        .set(SettingsScope::Project, &key, SettingValue::Int(11))
        .unwrap();
    registry
        .set(SettingsScope::Project, &key, SettingValue::Int(12))
        .unwrap();

    let delta = registry.changes_since(SettingsChangeCursor::origin());
    assert!(delta.requires_snapshot);
    assert!(delta.changes.is_empty());
}

#[test]
fn caught_up_cursor_does_not_require_a_snapshot_after_log_retention_expires() {
    let definition = project_grid_setting();
    let key = definition.key.clone();
    let mut registry = SettingsRegistry::with_change_log_policy(SettingsChangeLogPolicy::new(
        8,
        usize::MAX,
        Duration::ZERO,
    ));
    registry.register(definition).unwrap();

    registry
        .set(SettingsScope::Project, &key, SettingValue::Int(11))
        .unwrap();
    let cursor = registry.change_cursor();

    let delta = registry.changes_since(cursor);
    assert!(!delta.requires_snapshot);
    assert!(delta.changes.is_empty());
    assert_eq!(delta.cursor, cursor);
}

#[test]
fn authority_publishes_a_new_typed_snapshot_only_for_real_mutations() {
    let key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let authority = SettingsAuthority::with_defaults();
    let before = authority.snapshot();

    assert_eq!(before.generation(), 0);
    assert_eq!(before.viewport_snap().translate_step(), 1.0);

    let change = authority
        .set(SettingsScope::Project, &key, SettingValue::Float(2.5))
        .unwrap()
        .expect("a changed scoped value should publish a mutation");
    let after = authority.snapshot();

    assert_eq!(change.revision, 1);
    assert_eq!(after.generation(), 1);
    assert_eq!(before.viewport_snap().translate_step(), 1.0);
    assert_eq!(after.viewport_snap().translate_step(), 2.5);
    assert!(std::ptr::eq(before.design_tokens(), after.design_tokens()));
    assert!(std::ptr::eq(
        before.keymap_overrides(),
        after.keymap_overrides()
    ));
    assert!(std::ptr::eq(
        before.command_palette_mru(),
        after.command_palette_mru()
    ));
    assert_eq!(
        authority
            .changes_since(SettingsChangeCursor::origin())
            .changes,
        vec![change.clone()]
    );

    assert!(authority
        .set(SettingsScope::Project, &key, SettingValue::Float(2.5))
        .unwrap()
        .is_none());
    assert!(std::sync::Arc::ptr_eq(&after, &authority.snapshot()));
}

#[test]
fn authority_replaces_persistent_layers_without_republishing_identical_values() {
    let key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let authority = SettingsAuthority::with_defaults();
    let values = BTreeMap::from([(key, SettingValue::Float(3.0))]);

    let changes = authority
        .replace_persistent_layer(SettingsScope::Project, values.clone())
        .unwrap();
    let snapshot = authority.snapshot();

    assert_eq!(changes.len(), 1);
    assert_eq!(snapshot.generation(), 1);
    assert_eq!(snapshot.viewport_snap().translate_step(), 3.0);
    assert!(authority
        .replace_persistent_layer(SettingsScope::Project, values)
        .unwrap()
        .is_empty());
    assert!(std::sync::Arc::ptr_eq(&snapshot, &authority.snapshot()));
}

#[test]
fn project_layer_source_is_loaded_once_until_the_authority_generation_is_cleared() {
    let root = temporary_root("project-layer-cache");
    let project_root = root.join("project");
    let store = SettingsStore::from_roots(&root, Some(&project_root));
    let translate_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let mut source = settings_registry_with_defaults();
    source
        .set(
            SettingsScope::Project,
            &translate_key,
            SettingValue::Float(2.0),
        )
        .unwrap();
    store.save_from(SettingsScope::Project, &source).unwrap();

    let authority = SettingsAuthority::with_defaults();
    assert!(matches!(
        authority.load_project_layer_from_store(&store),
        SettingsProjectLayerLoad::Persisted {
            schema_version: 1,
            ..
        }
    ));
    assert_eq!(authority.snapshot().viewport_snap().translate_step(), 2.0);

    source
        .set(
            SettingsScope::Project,
            &translate_key,
            SettingValue::Float(4.0),
        )
        .unwrap();
    store.save_from(SettingsScope::Project, &source).unwrap();

    assert!(matches!(
        authority.load_project_layer_from_store(&store),
        SettingsProjectLayerLoad::Persisted {
            schema_version: 1,
            ..
        }
    ));
    assert_eq!(
        authority.snapshot().viewport_snap().translate_step(),
        2.0,
        "a repeated consumer must use the generation-bound project layer instead of rereading it"
    );

    authority.clear_project_layer();
    assert!(matches!(
        authority.load_project_layer_from_store(&store),
        SettingsProjectLayerLoad::Persisted {
            schema_version: 1,
            ..
        }
    ));
    assert_eq!(authority.snapshot().viewport_snap().translate_step(), 4.0);
    remove_temporary_root(&root);
}
