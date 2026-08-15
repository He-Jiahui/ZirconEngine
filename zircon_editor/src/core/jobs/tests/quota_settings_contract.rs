use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::jobs::{
    EDITOR_JOB_EXPORT_QUOTA_KEY, EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY, EDITOR_JOB_PLAY_QUOTA_KEY,
    EDITOR_JOB_THUMBNAIL_QUOTA_KEY, register_editor_job_quota_settings, resolve_editor_job_limits,
};
use crate::core::settings::{
    SettingValue, SettingsError, SettingsKey, SettingsRegistry, SettingsScope, SettingsStartup,
    SettingsStore, SettingsUserLayerLoad, settings_registry_with_defaults,
};

use super::super::JobCategory;

#[test]
fn job_category_quotas_are_user_scoped_bounded_and_restart_only() {
    let mut registry = SettingsRegistry::default();
    register_editor_job_quota_settings(&mut registry).unwrap();

    for key in [
        EDITOR_JOB_THUMBNAIL_QUOTA_KEY,
        EDITOR_JOB_EXPORT_QUOTA_KEY,
        EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY,
        EDITOR_JOB_PLAY_QUOTA_KEY,
    ] {
        let key = SettingsKey::parse(key).unwrap();
        let definition = registry.definition(&key).unwrap();
        assert_eq!(definition.scope, SettingsScope::User);
        assert!(definition.requires_restart);
        let change = registry
            .set(SettingsScope::User, &key, SettingValue::Int(2))
            .unwrap()
            .expect("a changed user quota must publish a settings change");
        assert!(change.requires_restart);
        assert!(
            registry
                .set(SettingsScope::User, &key, SettingValue::Int(0))
                .is_err()
        );
    }
}

#[test]
fn job_category_quota_presentations_resolve_for_every_embedded_locale() {
    let mut registry = SettingsRegistry::default();
    register_editor_job_quota_settings(&mut registry).unwrap();
    let catalog = crate::core::i18n::EditorI18nCatalog::embedded().unwrap();

    for key in [
        EDITOR_JOB_THUMBNAIL_QUOTA_KEY,
        EDITOR_JOB_EXPORT_QUOTA_KEY,
        EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY,
        EDITOR_JOB_PLAY_QUOTA_KEY,
    ] {
        let definition = registry
            .definition(&SettingsKey::parse(key).unwrap())
            .unwrap();
        let presentation = definition.presentation();
        for locale in catalog.available_locales() {
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
                assert_ne!(
                    catalog.translate_for_locale(&locale, category).as_ref(),
                    category,
                    "{key} has an unresolved category in {}",
                    locale.as_str()
                );
            }
        }
    }
}

#[test]
fn resolving_quotas_requires_the_registered_settings_definitions() {
    let error = resolve_editor_job_limits(&SettingsRegistry::default(), 4).unwrap_err();
    assert!(matches!(
        error,
        crate::core::jobs::EditorJobQuotaSettingsError::Settings(SettingsError::UnknownKey(_))
    ));
}

#[test]
fn resolved_quotas_override_only_the_user_configurable_categories() {
    let mut registry = SettingsRegistry::default();
    register_editor_job_quota_settings(&mut registry).unwrap();
    let thumbnail = SettingsKey::parse(EDITOR_JOB_THUMBNAIL_QUOTA_KEY).unwrap();
    registry
        .set(SettingsScope::User, &thumbnail, SettingValue::Int(3))
        .unwrap();

    let limits = resolve_editor_job_limits(&registry, 7).unwrap();
    assert_eq!(limits.limit(JobCategory::Thumbnail), 3);
    assert_eq!(limits.limit(JobCategory::Import), 7);
    assert_eq!(limits.limit(JobCategory::Compile), 7);
    assert_eq!(limits.limit(JobCategory::Index), 7);
    assert_eq!(limits.limit(JobCategory::Misc), 7);
}

#[test]
fn quota_settings_round_trip_through_the_current_user_store() {
    let root = temporary_root("quota-round-trip");
    let store = SettingsStore::from_roots(&root, None);
    let mut source = SettingsRegistry::default();
    register_editor_job_quota_settings(&mut source).unwrap();
    let export = SettingsKey::parse(EDITOR_JOB_EXPORT_QUOTA_KEY).unwrap();
    source
        .set(SettingsScope::User, &export, SettingValue::Int(4))
        .unwrap();
    store.save_from(SettingsScope::User, &source).unwrap();

    let mut restored = SettingsRegistry::default();
    register_editor_job_quota_settings(&mut restored).unwrap();
    store.load_into(SettingsScope::User, &mut restored).unwrap();
    let limits = resolve_editor_job_limits(&restored, 2).unwrap();
    assert_eq!(limits.limit(JobCategory::Export), 4);
    assert_eq!(limits.limit(JobCategory::Import), 2);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn settings_startup_preserves_every_registered_job_quota() {
    let root = temporary_root("production-registration");
    let startup = SettingsStartup::load_from_store(
        startup_registry(),
        &SettingsStore::from_roots(&root, None),
    );

    assert!(matches!(
        startup.user_layer_load(),
        SettingsUserLayerLoad::Missing { .. }
    ));
    for key in [
        EDITOR_JOB_THUMBNAIL_QUOTA_KEY,
        EDITOR_JOB_EXPORT_QUOTA_KEY,
        EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY,
        EDITOR_JOB_PLAY_QUOTA_KEY,
    ] {
        assert!(
            startup
                .registry()
                .definition(&SettingsKey::parse(key).unwrap())
                .is_some(),
            "production startup did not register {key}"
        );
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn startup_user_layer_reports_loaded_and_applies_runtime_derived_limits_once() {
    let root = temporary_root("loaded-runtime-defaults");
    let store = SettingsStore::from_roots(&root, None);
    let mut persisted = SettingsRegistry::default();
    register_editor_job_quota_settings(&mut persisted).unwrap();
    persisted
        .set(
            SettingsScope::User,
            &SettingsKey::parse(EDITOR_JOB_EXPORT_QUOTA_KEY).unwrap(),
            SettingValue::Int(3),
        )
        .unwrap();
    store.save_from(SettingsScope::User, &persisted).unwrap();

    let startup = SettingsStartup::load_from_store(startup_registry(), &store);
    assert!(matches!(
        startup.user_layer_load(),
        SettingsUserLayerLoad::Loaded { .. }
    ));
    let limits = resolve_editor_job_limits(startup.registry(), 7).unwrap();
    assert_eq!(limits.limit(JobCategory::Export), 3);
    for category in [
        JobCategory::Import,
        JobCategory::Compile,
        JobCategory::Index,
        JobCategory::Misc,
    ] {
        assert_eq!(limits.limit(category), 7, "{category:?}");
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn invalid_persisted_quota_is_typed_and_keeps_positive_defaults() {
    for (case, value) in [
        ("zero", serde_json::json!({ "kind": "int", "value": 0 })),
        (
            "negative",
            serde_json::json!({ "kind": "int", "value": -1 }),
        ),
        (
            "too-large",
            serde_json::json!({ "kind": "int", "value": 65 }),
        ),
        (
            "wrong-kind",
            serde_json::json!({ "kind": "bool", "value": true }),
        ),
    ] {
        let root = temporary_root(case);
        let store = SettingsStore::from_roots(&root, None);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            store.paths().user(),
            serde_json::to_vec(&serde_json::json!({
                "$zircon": {
                    "header": {
                        "schema_id": "zircon.editor.settings",
                        "schema_version": 1
                    },
                    "payload": {
                        "values": {
                            (EDITOR_JOB_EXPORT_QUOTA_KEY): value
                        }
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let startup = SettingsStartup::load_from_store(startup_registry(), &store);
        assert!(matches!(
            startup.user_layer_load(),
            SettingsUserLayerLoad::Invalid { .. }
        ));
        let limits = resolve_editor_job_limits(startup.registry(), 4).unwrap();
        assert_eq!(limits.limit(JobCategory::Export), 1);
        assert!(
            JobCategory::ALL
                .into_iter()
                .all(|category| limits.limit(category) > 0),
            "{case} admitted a zero-capacity category"
        );

        let _ = fs::remove_dir_all(root);
    }
}

fn temporary_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-editor-job-quota-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos()
    ))
}

fn startup_registry() -> SettingsRegistry {
    let mut registry = settings_registry_with_defaults();
    register_editor_job_quota_settings(&mut registry).unwrap();
    registry
}
