use std::collections::BTreeMap;

use crate::core::commands::EditorKeyChord;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::settings::{
    EditorKeymapOverrides, SettingValue, SettingsAuthority, SettingsKey, SettingsScope,
    EDITOR_KEYMAP_OVERRIDES_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
};

use super::EditorKeymapProjection;

#[test]
fn keymap_projection_reuses_the_authority_payload_until_overrides_change() {
    let authority = SettingsAuthority::with_defaults();
    let initial = authority.snapshot();
    let mut projection = EditorKeymapProjection::from_snapshot(initial.as_ref());
    assert!(!projection.refresh_if_changed(initial.as_ref()));

    let viewport_key = SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
    authority
        .set(
            SettingsScope::Project,
            &viewport_key,
            SettingValue::Float(2.0),
        )
        .unwrap();
    assert!(!projection.refresh_if_changed(authority.snapshot().as_ref()));

    let keymap_key = SettingsKey::parse(EDITOR_KEYMAP_OVERRIDES_KEY).unwrap();
    let overrides = EditorKeymapOverrides::new(BTreeMap::from([(
        EditorOperationPath::parse("file.project.open").unwrap(),
        Some("Alt+O".parse::<EditorKeyChord>().unwrap()),
    )]));
    authority
        .set(
            SettingsScope::User,
            &keymap_key,
            SettingValue::KeymapOverrides(overrides),
        )
        .unwrap();
    assert!(projection.refresh_if_changed(authority.snapshot().as_ref()));
    assert_eq!(
        projection
            .keymap
            .chord_for_command("file.project.open")
            .unwrap()
            .to_string(),
        "Alt+O"
    );
}

#[test]
fn manager_keyboard_dispatch_uses_the_contextual_keymap_resolver() {
    let source = include_str!("../editor_manager.rs");
    let shared_snapshot = ["shared_", "snapshot()"].concat();
    let contextual_resolver = ["resolve_keyboard_", "input_when(keyboard"].concat();
    let chord_only_resolver = ["self.keymap.", "resolve_keyboard_input(keyboard)"].concat();

    assert!(source.contains(&shared_snapshot));
    assert!(source.contains(&contextual_resolver));
    assert!(!source.contains(&chord_only_resolver));
}
