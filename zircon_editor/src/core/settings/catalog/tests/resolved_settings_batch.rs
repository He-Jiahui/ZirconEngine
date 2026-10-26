use super::*;
use crate::core::settings::{SettingsAuthority, VIEWPORT_TRANSLATE_STEP_KEY};

fn key() -> SettingsKey {
    SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap()
}

#[test]
fn selected_batch_preserves_hidden_project_and_user_overrides() {
    let authority = SettingsAuthority::with_defaults();
    let key = key();
    for (scope, value) in [
        (SettingsScope::User, 2.5),
        (SettingsScope::Project, 5.0),
        (SettingsScope::Session, 7.5),
    ] {
        authority
            .set(scope, &key, SettingValue::Float(value))
            .unwrap();
    }
    let batch = authority
        .resolved_settings(std::slice::from_ref(&key))
        .unwrap();
    let row = &batch.values()[0];
    assert_eq!(row.key(), &key);
    assert_eq!(row.value(), &SettingValue::Float(7.5));
    assert_eq!(
        row.source(),
        SettingValueSource::Scope(SettingsScope::Session)
    );
    assert_eq!(
        row.override_in_scope(SettingsScope::User),
        Some(&SettingValue::Float(2.5))
    );
    assert_eq!(
        row.override_in_scope(SettingsScope::Project),
        Some(&SettingValue::Float(5.0))
    );
    assert_eq!(
        row.override_in_scope(SettingsScope::Session),
        Some(&SettingValue::Float(7.5))
    );
    assert_eq!(batch.generation(), authority.snapshot().generation());
}

#[test]
fn inherited_value_does_not_become_a_project_or_session_override() {
    let authority = SettingsAuthority::with_defaults();
    let key = key();
    let defaults = authority
        .resolved_settings(std::slice::from_ref(&key))
        .unwrap();
    assert_eq!(defaults.values()[0].source(), SettingValueSource::Default);
    for scope in [
        SettingsScope::User,
        SettingsScope::Project,
        SettingsScope::Session,
    ] {
        assert_eq!(defaults.values()[0].override_in_scope(scope), None);
    }
    authority
        .set(SettingsScope::User, &key, SettingValue::Float(2.5))
        .unwrap();
    let inherited = authority
        .resolved_settings(std::slice::from_ref(&key))
        .unwrap();
    let row = &inherited.values()[0];
    assert_eq!(row.value(), &SettingValue::Float(2.5));
    assert_eq!(row.source(), SettingValueSource::Scope(SettingsScope::User));
    assert_eq!(
        row.override_in_scope(SettingsScope::User),
        Some(&SettingValue::Float(2.5))
    );
    assert_eq!(row.override_in_scope(SettingsScope::Project), None);
    assert_eq!(row.override_in_scope(SettingsScope::Session), None);
    assert!(authority
        .clear(SettingsScope::Project, &key)
        .unwrap()
        .is_none());
    assert_eq!(inherited.generation(), authority.snapshot().generation());
}

#[test]
fn captured_raw_layers_stay_immutable_when_a_hidden_override_is_cleared() {
    let authority = SettingsAuthority::with_defaults();
    let key = key();
    authority
        .set(SettingsScope::User, &key, SettingValue::Float(2.5))
        .unwrap();
    authority
        .set(SettingsScope::Project, &key, SettingValue::Float(5.0))
        .unwrap();
    authority
        .set(SettingsScope::Session, &key, SettingValue::Float(7.5))
        .unwrap();
    let before = authority
        .resolved_settings(std::slice::from_ref(&key))
        .unwrap();
    assert!(authority
        .clear(SettingsScope::Project, &key)
        .unwrap()
        .is_some());
    let after = authority
        .resolved_settings(std::slice::from_ref(&key))
        .unwrap();
    assert!(before.generation() < after.generation());
    assert_eq!(
        before.values()[0].override_in_scope(SettingsScope::Project),
        Some(&SettingValue::Float(5.0))
    );
    assert_eq!(
        after.values()[0].override_in_scope(SettingsScope::Project),
        None
    );
    assert_eq!(before.values()[0].value(), after.values()[0].value());
    assert_eq!(
        after.values()[0].source(),
        SettingValueSource::Scope(SettingsScope::Session)
    );
    assert_eq!(
        after.values()[0].override_in_scope(SettingsScope::User),
        Some(&SettingValue::Float(2.5))
    );
    assert_eq!(after.generation(), authority.snapshot().generation());
}
