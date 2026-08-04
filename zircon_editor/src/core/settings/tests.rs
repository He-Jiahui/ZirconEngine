use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use zircon_runtime_interface::serialization::write_versioned_text;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use super::defaults::{
    EDITOR_COMMAND_PALETTE_MRU_KEY, EDITOR_DESIGN_TOKENS_KEY, VIEWPORT_ROTATE_STEP_DEGREES_KEY,
    VIEWPORT_SCALE_STEP_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
};
use super::io::SettingsDocument;
use super::{
    EditorCommandPaletteMru, SettingDefinition, SettingSchema, SettingValue, SettingsAuthority,
    SettingsChangeCursor, SettingsChangeLogPolicy, SettingsChangeSubscriber, SettingsDecodeError,
    SettingsError, SettingsKey, SettingsLoad, SettingsPaths, SettingsPersistenceLimits,
    SettingsPersistenceService, SettingsPersistenceShutdownError, SettingsProjectLayerLoad,
    SettingsRegistry, SettingsScope, SettingsStore, SettingsStoreError,
    settings_registry_with_defaults,
};
use crate::core::editor_operation::EditorOperationPath;

fn key(value: &str) -> SettingsKey {
    SettingsKey::parse(value).unwrap()
}

fn project_grid_setting() -> SettingDefinition {
    SettingDefinition::new(
        key("editor.scene.grid_step"),
        SettingsScope::Project,
        SettingSchema::Int {
            minimum: 1,
            maximum: 100,
        },
        SettingValue::Int(10),
        false,
        "Scene/Grid",
    )
    .unwrap()
}

struct ReentrantSettingsChangeSubscriber {
    authority: Arc<SettingsAuthority>,
    observed: Sender<(usize, u64, usize)>,
}

impl SettingsChangeSubscriber for ReentrantSettingsChangeSubscriber {
    fn settings_changed(
        &self,
        changes: &[super::SettingChange],
        snapshot: &super::SettingsSnapshot,
    ) {
        let delta = self.authority.changes_since(SettingsChangeCursor::origin());
        let _ = self
            .observed
            .send((changes.len(), snapshot.generation(), delta.changes.len()));
    }
}

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
    let change = super::SettingChange {
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
    assert!(
        authority
            .record_command_palette_usage(
                EditorOperationPath::parse("editor.command_palette.open").unwrap()
            )
            .unwrap()
            .is_none()
    );
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
        "Editor/Autosave",
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
    assert_eq!(delta.changes, vec![second, third]);
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

    assert!(
        authority
            .set(SettingsScope::Project, &key, SettingValue::Float(2.5))
            .unwrap()
            .is_none()
    );
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
    assert!(
        authority
            .replace_persistent_layer(SettingsScope::Project, values)
            .unwrap()
            .is_empty()
    );
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

#[test]
fn invalid_keys_and_duplicate_definitions_are_rejected() {
    assert!(SettingsKey::parse("Editor.scene.grid_step").is_err());
    assert!(SettingsKey::parse("editor..grid_step").is_err());

    let definition = project_grid_setting();
    let mut registry = SettingsRegistry::default();
    registry.register(definition.clone()).unwrap();
    assert!(matches!(
        registry.register(definition),
        Err(SettingsError::DuplicateDefinition(_))
    ));
}

#[test]
fn direct_definition_literals_are_revalidated_at_registration() {
    let definition = SettingDefinition {
        key: key("editor.invalid.float"),
        scope: SettingsScope::User,
        schema: SettingSchema::Float {
            minimum: f64::NAN,
            maximum: 1.0,
        },
        default: SettingValue::Float(0.5),
        requires_restart: false,
        category_path: String::new(),
    };
    let mut registry = SettingsRegistry::default();

    assert!(matches!(
        registry.register(definition),
        Err(SettingsError::InvalidDefinition { .. })
    ));

    let inverted = SettingDefinition {
        key: key("editor.invalid.range"),
        scope: SettingsScope::Project,
        schema: SettingSchema::Int {
            minimum: 10,
            maximum: 1,
        },
        default: SettingValue::Int(5),
        requires_restart: false,
        category_path: "Editor/Invalid".to_string(),
    };
    assert!(matches!(
        registry.register(inverted),
        Err(SettingsError::InvalidDefinition { .. })
    ));
}

#[test]
fn scope_and_schema_boundaries_remain_explicit() {
    assert!(SettingsScope::User.allows_write(SettingsScope::User));
    assert!(SettingsScope::User.allows_write(SettingsScope::Session));
    assert!(!SettingsScope::User.allows_write(SettingsScope::Project));
    assert!(SettingsScope::Project.allows_write(SettingsScope::User));
    assert!(SettingsScope::Project.allows_write(SettingsScope::Project));
    assert!(SettingsScope::Project.allows_write(SettingsScope::Session));
    assert!(!SettingsScope::Session.allows_write(SettingsScope::User));
    assert!(!SettingsScope::Session.allows_write(SettingsScope::Project));
    assert!(SettingsScope::Session.allows_write(SettingsScope::Session));

    let string = SettingDefinition::new(
        key("editor.caption"),
        SettingsScope::User,
        SettingSchema::String { maximum_bytes: 3 },
        SettingValue::String("abc".to_string()),
        false,
        "Editor/Caption",
    )
    .unwrap();
    let chord = SettingDefinition::new(
        key("editor.shortcut"),
        SettingsScope::User,
        SettingSchema::Chord,
        SettingValue::Chord("Ctrl+S".to_string()),
        false,
        "Editor/Shortcut",
    )
    .unwrap();
    let mut registry = SettingsRegistry::default();
    let string_key = string.key.clone();
    registry.register(string).unwrap();
    registry.register(chord).unwrap();

    assert!(matches!(
        registry.set(
            SettingsScope::User,
            &string_key,
            SettingValue::String("four".to_string())
        ),
        Err(SettingsError::InvalidValue { .. })
    ));
    assert!(matches!(
        registry.set(
            SettingsScope::User,
            &key("editor.shortcut"),
            SettingValue::Chord("   ".to_string())
        ),
        Err(SettingsError::InvalidValue { .. })
    ));
    assert!(matches!(
        registry.clear(SettingsScope::User, &key("editor.unknown")),
        Err(SettingsError::UnknownKey(_))
    ));
}

#[test]
fn design_tokens_are_a_strongly_typed_user_setting() {
    let mut registry = settings_registry_with_defaults();
    let key = key(EDITOR_DESIGN_TOKENS_KEY);
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.id = "zircon.editor.tests.custom".to_string();
    tokens.density.row_height = 31.0;

    registry
        .set(
            SettingsScope::User,
            &key,
            SettingValue::DesignTokens(tokens.clone()),
        )
        .unwrap();
    assert_eq!(
        registry.resolve(&key).unwrap(),
        &SettingValue::DesignTokens(tokens)
    );
    assert!(matches!(
        registry.set(
            SettingsScope::User,
            &key,
            SettingValue::String("wrong".to_string())
        ),
        Err(SettingsError::InvalidValue { .. })
    ));
}

#[test]
fn command_palette_mru_is_a_bounded_session_only_setting() {
    let mut registry = settings_registry_with_defaults();
    let key = key(EDITOR_COMMAND_PALETTE_MRU_KEY);
    let mru = EditorCommandPaletteMru::new([
        EditorOperationPath::parse("file.project.open")
            .expect("the built-in command id should be valid"),
        EditorOperationPath::parse("file.project.save")
            .expect("the built-in command id should be valid"),
    ])
    .expect("the bounded command history should be valid");

    assert!(matches!(
        registry.set(
            SettingsScope::User,
            &key,
            SettingValue::CommandPaletteMru(mru.clone()),
        ),
        Err(SettingsError::ScopeNotAllowed { .. })
    ));
    registry
        .set(
            SettingsScope::Session,
            &key,
            SettingValue::CommandPaletteMru(mru.clone()),
        )
        .expect("the Session layer should own command palette history");
    assert_eq!(
        registry
            .resolve(&key)
            .expect("the MRU setting should resolve"),
        &SettingValue::CommandPaletteMru(mru),
    );
}

#[test]
fn viewport_snap_steps_resolve_at_project_scope_and_round_trip_without_touching_project_sources() {
    let root = temporary_root("viewport-snap-steps");
    let project_root = root.join("project");
    let source_path = project_root
        .join("assets")
        .join("scenes")
        .join("main.zscene");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "scene source stays outside editor settings\n").unwrap();
    let source_digest_before = blake3::hash(&fs::read(&source_path).unwrap());

    let store = SettingsStore::from_roots(&root, Some(&project_root));
    let translate_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    let rotate_key = key(VIEWPORT_ROTATE_STEP_DEGREES_KEY);
    let scale_key = key(VIEWPORT_SCALE_STEP_KEY);
    let mut source = settings_registry_with_defaults();
    source
        .set(
            SettingsScope::User,
            &translate_key,
            SettingValue::Float(0.5),
        )
        .unwrap();
    source
        .set(SettingsScope::User, &rotate_key, SettingValue::Float(30.0))
        .unwrap();
    source
        .set(
            SettingsScope::Project,
            &translate_key,
            SettingValue::Float(2.0),
        )
        .unwrap();
    source
        .set(
            SettingsScope::Project,
            &scale_key,
            SettingValue::Float(0.25),
        )
        .unwrap();
    store.save_from(SettingsScope::User, &source).unwrap();
    store.save_from(SettingsScope::Project, &source).unwrap();

    let encoded = fs::read_to_string(store.paths().project().unwrap()).unwrap();
    assert!(encoded.contains(VIEWPORT_TRANSLATE_STEP_KEY));
    assert!(encoded.contains(VIEWPORT_SCALE_STEP_KEY));

    let mut restored = settings_registry_with_defaults();
    store.load_into(SettingsScope::User, &mut restored).unwrap();
    store
        .load_into(SettingsScope::Project, &mut restored)
        .unwrap();
    assert_eq!(
        restored.resolve(&translate_key).unwrap(),
        &SettingValue::Float(2.0)
    );
    assert_eq!(
        restored.resolve(&rotate_key).unwrap(),
        &SettingValue::Float(30.0)
    );
    assert_eq!(
        restored.resolve(&scale_key).unwrap(),
        &SettingValue::Float(0.25)
    );

    restored
        .set(
            SettingsScope::Session,
            &translate_key,
            SettingValue::Float(4.0),
        )
        .unwrap();
    assert_eq!(
        restored.resolve(&translate_key).unwrap(),
        &SettingValue::Float(4.0)
    );

    assert_eq!(
        source_digest_before,
        blake3::hash(&fs::read(&source_path).unwrap())
    );
    remove_temporary_root(&root);
}

#[test]
fn settings_store_round_trips_current_shell_at_planned_user_and_project_paths() {
    let root = temporary_root("round-trip");
    let project_root = root.join("project");
    let store = SettingsStore::from_roots(&root, Some(&project_root));
    let expected_user_settings = root.join("settings.toml");
    let expected_project_settings = project_root.join(".zircon").join("settings.toml");
    assert_eq!(store.paths().user(), expected_user_settings.as_path());
    assert_eq!(
        store.paths().project(),
        Some(expected_project_settings.as_path())
    );

    let settings_key = key(EDITOR_DESIGN_TOKENS_KEY);
    let mut expected = EditorDesignTokens::workbench_dark();
    expected.id = "zircon.editor.tests.persisted".to_string();
    expected.controls.default_height = 37.0;
    let mut source_registry = settings_registry_with_defaults();
    source_registry
        .set(
            SettingsScope::User,
            &settings_key,
            SettingValue::DesignTokens(expected.clone()),
        )
        .unwrap();
    store
        .save_from(SettingsScope::User, &source_registry)
        .unwrap();

    let encoded = fs::read_to_string(store.paths().user()).unwrap();
    assert!(encoded.contains("\"$zircon\""));
    assert!(encoded.contains("\"schema_id\": \"zircon.editor.settings\""));
    assert!(encoded.ends_with('\n'));

    let mut restored = settings_registry_with_defaults();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut restored).unwrap(),
        SettingsLoad::Loaded { .. }
    ));
    assert_eq!(
        restored.resolve(&settings_key).unwrap(),
        &SettingValue::DesignTokens(expected)
    );

    let mut replacement = EditorDesignTokens::workbench_dark();
    replacement.id = "zircon.editor.tests.replaced".to_string();
    source_registry
        .set(
            SettingsScope::User,
            &settings_key,
            SettingValue::DesignTokens(replacement.clone()),
        )
        .unwrap();
    store
        .save_from(SettingsScope::User, &source_registry)
        .unwrap();
    let mut replaced = settings_registry_with_defaults();
    store.load_into(SettingsScope::User, &mut replaced).unwrap();
    assert_eq!(
        replaced.resolve(&settings_key).unwrap(),
        &SettingValue::DesignTokens(replacement)
    );
    remove_temporary_root(&root);
}

#[test]
fn settings_store_rejects_retired_formats_and_keeps_the_existing_layer_atomic() {
    let root = temporary_root("strict-load");
    let store = SettingsStore::from_roots(&root, None);
    let settings_key = key(EDITOR_DESIGN_TOKENS_KEY);
    fs::create_dir_all(root.as_path()).unwrap();
    fs::write(store.paths().user(), "active_profile = 'legacy'\n").unwrap();
    let mut registry = settings_registry_with_defaults();

    assert!(matches!(
        store.load_into(SettingsScope::User, &mut registry),
        Err(SettingsStoreError::Decode {
            source: SettingsDecodeError::LegacyPayload,
            ..
        })
    ));
    let default = registry.resolve(&settings_key).unwrap().clone();

    let mut custom_tokens = EditorDesignTokens::workbench_dark();
    custom_tokens.id = "zircon.editor.tests.rejected".to_string();
    let invalid_document = SettingsDocument {
        values: BTreeMap::from([
            (
                settings_key.clone(),
                SettingValue::DesignTokens(custom_tokens),
            ),
            (key("editor.unknown_setting"), SettingValue::Bool(true)),
        ]),
    };
    fs::write(
        store.paths().user(),
        write_versioned_text(&invalid_document).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut registry),
        Err(SettingsStoreError::Apply {
            source: SettingsError::UnknownKey(_),
            ..
        })
    ));
    assert_eq!(registry.resolve(&settings_key).unwrap(), &default);

    let current = fs::read_to_string(store.paths().user()).unwrap();
    let mut retired = serde_json::from_str::<serde_json::Value>(&current).unwrap();
    retired["$zircon"]["header"]["schema_version"] = json!(0);
    fs::write(
        store.paths().user(),
        serde_json::to_string(&retired).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut registry),
        Err(SettingsStoreError::Decode {
            source: SettingsDecodeError::Versioned(_),
            ..
        })
    ));

    let malformed_key = serde_json::json!({
        "$zircon": {
            "header": {
                "schema_id": "zircon.editor.settings",
                "schema_version": 1
            },
            "payload": {
                "values": {
                    "Editor.invalid": { "kind": "bool", "value": true }
                }
            }
        }
    });
    fs::write(
        store.paths().user(),
        serde_json::to_string(&malformed_key).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut registry),
        Err(SettingsStoreError::Decode {
            source: SettingsDecodeError::Versioned(_),
            ..
        })
    ));
    remove_temporary_root(&root);
}

#[test]
fn user_environment_value_is_a_root_override_not_a_retired_file_override() {
    let root = temporary_root("env-root");
    let root_value = OsString::from(root.as_os_str());
    assert_eq!(
        SettingsPaths::user_root_from_env_value(Some(root_value)).unwrap(),
        root
    );
    let expected_user_settings = root.join("settings.toml");
    assert_eq!(
        SettingsPaths::from_roots(&root, None).user(),
        expected_user_settings.as_path()
    );

    fs::write(&root, "retired settings file").unwrap();
    assert!(matches!(
        SettingsPaths::user_root_from_env_value(Some(OsString::from(root.as_os_str()))),
        Err(SettingsStoreError::UserRootIsFile { .. })
    ));
    let _ = fs::remove_file(root);
}

#[test]
fn session_layer_cannot_be_persisted() {
    let root = temporary_root("session-only");
    let store = SettingsStore::from_roots(&root, None);
    let registry = settings_registry_with_defaults();
    assert!(matches!(
        store.save_from(SettingsScope::Session, &registry),
        Err(SettingsStoreError::NonPersistentScope(
            SettingsScope::Session
        ))
    ));
}

fn temporary_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-editor-settings-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos()
    ))
}

fn remove_temporary_root(path: &Path) {
    let _ = fs::remove_dir_all(path);
}
