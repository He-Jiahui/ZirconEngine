use super::*;
use crate::core::settings::{
    SettingsMutationCoordinator, EDITOR_AUTOSAVE_INTERVAL_SECS_KEY, EDITOR_LOCALE_KEY,
    EDITOR_SHOW_PREVIEW_FEATURES_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
};
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn popup_open_requires_same_batch_matching_value_kind_without_replacing_an_open_editor() {
    let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .open_settings_window(guarded_preferences_tests::open_state(
            &mutations,
            "builtin|settings.category.editor",
        ))
        .unwrap();
    let displayed = bridge.settings_values_context().unwrap().clone();
    assert!(bridge
        .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
        .unwrap());
    for (key, kind) in [
        (EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Color),
        (
            EDITOR_SHOW_PREVIEW_FEATURES_KEY,
            WorkbenchSettingsEditorKind::Enum,
        ),
        (
            EDITOR_AUTOSAVE_INTERVAL_SECS_KEY,
            WorkbenchSettingsEditorKind::Enum,
        ),
        (
            VIEWPORT_TRANSLATE_STEP_KEY,
            WorkbenchSettingsEditorKind::Enum,
        ),
        ("editor.missing.popup", WorkbenchSettingsEditorKind::Enum),
    ] {
        assert!(!bridge.toggle_settings_editor(key, kind).unwrap());
        assert_eq!(
            bridge
                .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
                .as_deref(),
            Some(EDITOR_LOCALE_KEY)
        );
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
    }
    mutations.shutdown().wait();
}

#[test]
fn popup_close_survives_context_invalidation_but_reopen_requires_the_displayed_category() {
    let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .open_settings_window(guarded_preferences_tests::open_state(
            &mutations,
            "builtin|settings.category.editor",
        ))
        .unwrap();
    assert!(bridge
        .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
        .unwrap());
    bridge.invalidate_settings_values_context();
    assert!(bridge
        .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
        .unwrap());
    assert!(bridge
        .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
        .is_none());
    assert!(!bridge
        .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
        .unwrap());
    let locale = SettingsKey::parse(EDITOR_LOCALE_KEY).unwrap();
    bridge
        .select_settings_category(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            "builtin|settings.category.editor/settings.category.language",
            &mutations.capture_values([&locale]).unwrap(),
        )
        .unwrap();
    assert!(bridge
        .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
        .unwrap());
    let autosave = SettingsKey::parse(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY).unwrap();
    bridge
        .select_settings_category(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            "builtin|settings.category.editor/settings.category.autosave",
            &mutations.capture_values([&autosave]).unwrap(),
        )
        .unwrap();
    assert!(bridge
        .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
        .is_none());
    assert!(!bridge
        .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
        .unwrap());
    assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
    mutations.shutdown().wait();
}
