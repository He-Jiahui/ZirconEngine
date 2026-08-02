use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::jobs::{
    EDITOR_JOB_EXPORT_QUOTA_KEY, EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY, EDITOR_JOB_PLAY_QUOTA_KEY,
    EDITOR_JOB_THUMBNAIL_QUOTA_KEY, register_editor_job_quota_settings, resolve_editor_job_limits,
};
use crate::core::settings::{
    SettingValue, SettingsError, SettingsKey, SettingsRegistry, SettingsScope, SettingsStore,
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
            .unwrap();
        assert!(change.requires_restart);
        assert!(
            registry
                .set(SettingsScope::User, &key, SettingValue::Int(0))
                .is_err()
        );
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
