use crate::core::settings::{
    SettingsAuthority, EDITOR_AUTOSAVE_INTERVAL_SECS_KEY, EDITOR_LOCALE_KEY,
};

#[test]
fn parent_builtin_category_resolves_descendant_values() {
    let authority = SettingsAuthority::with_defaults();
    let snapshot = authority.snapshot();
    let editor = snapshot
        .catalog()
        .keys_for_category_subtree("settings.category.editor")
        .collect::<Vec<_>>();
    let values = authority
        .resolved_settings_from_iter(editor.iter().copied())
        .unwrap();

    assert!(editor.iter().any(|key| key.as_str() == EDITOR_LOCALE_KEY));
    assert!(editor
        .iter()
        .any(|key| key.as_str() == EDITOR_AUTOSAVE_INTERVAL_SECS_KEY));
    assert_eq!(values.values().len(), editor.len());

    let autosave = snapshot
        .catalog()
        .keys_for_category_subtree("settings.category.editor/settings.category.autosave")
        .collect::<Vec<_>>();
    assert_eq!(autosave.len(), 1);
    assert_eq!(autosave[0].as_str(), EDITOR_AUTOSAVE_INTERVAL_SECS_KEY);
    assert!(snapshot
        .catalog()
        .keys_for_category_subtree("settings.category.edit")
        .next()
        .is_none());
}

#[test]
fn guarded_preferences_category_capture_keeps_keys_and_binding_with_the_displayed_generation() {
    use crate::core::settings::{
        SettingValue, SettingsKey, SettingsMutationCoordinator, SettingsScope,
    };
    let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
    let displayed = super::capture_values_for_category(
        &mutations,
        "builtin|settings.category.editor/settings.category.autosave",
    )
    .unwrap();
    assert_eq!(displayed.values().values().len(), 1);
    let key = SettingsKey::parse(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY).unwrap();
    assert!(displayed.value(&key).is_some());
    assert!(displayed
        .value(&SettingsKey::parse(EDITOR_LOCALE_KEY).unwrap())
        .is_none());
    assert!(mutations.context_is_current(&displayed));
    mutations
        .authority()
        .set(SettingsScope::Session, &key, SettingValue::Int(45))
        .unwrap();
    assert!(!mutations.context_is_current(&displayed));
    let refreshed = super::capture_values_for_category(
        &mutations,
        "builtin|settings.category.editor/settings.category.autosave",
    )
    .unwrap();
    assert_eq!(
        refreshed.generation(),
        mutations.authority().snapshot().generation()
    );
    assert!(refreshed.generation() > displayed.generation());
    assert_eq!(
        refreshed.value(&key).unwrap().value(),
        &SettingValue::Int(45)
    );
}

#[test]
fn guarded_preferences_plugin_and_invalid_categories_do_not_capture_builtin_edit_keys() {
    let mutations = crate::core::settings::SettingsMutationCoordinator::in_memory_with_defaults();
    for category in [
        "",
        "plugin.test|settings.category.editor",
        "builtin|",
        "builtin|settings.category.edit",
    ] {
        let displayed = super::capture_values_for_category(&mutations, category).unwrap();
        assert!(displayed.values().is_empty(), "{category}");
        assert!(mutations.context_is_current(&displayed));
    }
}

fn projection_for_values_capture(
    mutations: &crate::core::settings::SettingsMutationCoordinator,
) -> crate::ui::settings::SettingsWindowProjection {
    crate::ui::settings::SettingsWindowProjection::capture(
        &mutations.authority().snapshot(),
        &crate::core::extension::ContributionStore::default().snapshot(),
        &crate::core::extension::CapabilitySet::default(),
        &crate::core::i18n::EditorI18nService::default(),
    )
}

#[test]
fn projection_values_capture_pairs_builtin_category_with_the_original_projection() {
    let mutations = crate::core::settings::SettingsMutationCoordinator::in_memory_with_defaults();
    let projection = projection_for_values_capture(&mutations);
    let captured = super::capture_values_for_projection_category(
        &mutations,
        &projection,
        "builtin|settings.category.editor/settings.category.autosave",
    )
    .unwrap();
    assert_eq!(captured.generation(), projection.settings_generation());
    assert_eq!(captured.values().values().len(), 1);
    let autosave =
        crate::core::settings::SettingsKey::parse(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY).unwrap();
    assert!(captured.value(&autosave).is_some());
    assert!(captured
        .value(&crate::core::settings::SettingsKey::parse(EDITOR_LOCALE_KEY).unwrap())
        .is_none());
    assert!(mutations.context_is_current(&captured));
    mutations.shutdown().wait();
}

#[test]
fn projection_values_capture_rejects_stale_builtin_and_empty_categories_until_recaptured() {
    use crate::core::settings::{
        SettingValue, SettingsError, SettingsKey, SettingsMutationError, SettingsScope,
    };
    let mutations = crate::core::settings::SettingsMutationCoordinator::in_memory_with_defaults();
    let projection = projection_for_values_capture(&mutations);
    let autosave = SettingsKey::parse(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY).unwrap();
    mutations
        .authority()
        .set(SettingsScope::Session, &autosave, SettingValue::Int(45))
        .unwrap();
    let actual = mutations.authority().snapshot().generation();
    let health = mutations.persistence_health_snapshot();
    let categories = [
        "builtin|settings.category.editor/settings.category.autosave",
        "plugin.test|settings.category.editor",
        "builtin|",
        "builtin|settings.category.edit",
        "",
    ];
    for category in categories {
        assert_eq!(
            super::capture_values_for_projection_category(&mutations, &projection, category),
            Err(SettingsMutationError::Settings(
                SettingsError::StaleGeneration {
                    expected: projection.settings_generation(),
                    actual,
                }
            )),
            "{category}"
        );
    }
    assert_eq!(mutations.persistence_health_snapshot(), health);
    assert_eq!(mutations.authority().snapshot().generation(), actual);
    let refreshed_projection = projection_for_values_capture(&mutations);
    for (index, category) in categories.iter().enumerate() {
        let captured = super::capture_values_for_projection_category(
            &mutations,
            &refreshed_projection,
            category,
        )
        .unwrap();
        assert_eq!(
            captured.generation(),
            refreshed_projection.settings_generation()
        );
        assert!(mutations.context_is_current(&captured));
        if index == 0 {
            assert_eq!(
                captured.value(&autosave).unwrap().value(),
                &SettingValue::Int(45)
            );
            assert_eq!(captured.values().values().len(), 1);
        } else {
            assert!(captured.values().is_empty(), "{category}");
        }
    }
    mutations.shutdown().wait();
}
