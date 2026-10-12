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
