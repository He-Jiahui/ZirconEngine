use super::*;

#[test]
fn category_index_drives_one_generation_consistent_value_batch() {
    let authority = SettingsAuthority::with_defaults();
    let snapshot = authority.snapshot();
    let keys = snapshot
        .catalog()
        .keys_for_category_path("settings.category.viewport/settings.category.snapping");

    assert_eq!(keys.len(), 3);
    let before = authority.resolved_settings(keys).unwrap();
    assert_eq!(before.generation(), snapshot.generation());
    assert_eq!(before.values().len(), keys.len());
    assert!(before.values().iter().all(|value| {
        value.source() == SettingValueSource::Default && keys.contains(value.key())
    }));

    let translate_key = key(VIEWPORT_TRANSLATE_STEP_KEY);
    authority
        .set(
            SettingsScope::Project,
            &translate_key,
            SettingValue::Float(2.5),
        )
        .unwrap();
    let after = authority.resolved_settings(keys).unwrap();
    let translate = after
        .values()
        .iter()
        .find(|value| value.key() == &translate_key)
        .unwrap();

    assert_eq!(after.generation(), authority.snapshot().generation());
    assert_eq!(translate.value(), &SettingValue::Float(2.5));
    assert_eq!(
        translate.source(),
        SettingValueSource::Scope(SettingsScope::Project)
    );
}

#[test]
fn value_batch_rejects_an_unknown_key_without_partial_output() {
    let authority = SettingsAuthority::with_defaults();
    let unknown = key("editor.fixture.missing");

    assert!(matches!(
        authority.resolved_settings(std::slice::from_ref(&unknown)),
        Err(SettingsError::UnknownKey(key)) if key == unknown.as_str()
    ));
}
