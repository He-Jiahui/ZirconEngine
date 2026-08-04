use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::ui::dispatch::{
    UiInputEventMetadata, UiInputModifiers, UiInputSequence, UiInputTimestamp,
    UiKeyboardInputEvent, UiKeyboardInputState,
};

use crate::core::editor_operation::EditorOperationPath;
use crate::core::settings::{
    EDITOR_KEYMAP_OVERRIDES_KEY, EditorKeymapOverrides, SettingValue, SettingsDecodeError,
    SettingsKey, SettingsLoad, SettingsScope, SettingsStore, SettingsStoreError,
    editor_keymap_overrides, settings_registry_with_defaults,
};

use super::{EditorKeyChord, EditorKeymap};

#[test]
fn user_and_session_keymap_overrides_resolve_through_settings_registry() {
    let key = SettingsKey::parse(EDITOR_KEYMAP_OVERRIDES_KEY).unwrap();
    let mut settings = settings_registry_with_defaults();
    settings
        .set(
            SettingsScope::User,
            &key,
            SettingValue::KeymapOverrides(overrides([("file.project.open", Some("Ctrl+Shift+O"))])),
        )
        .unwrap();

    let user_keymap =
        EditorKeymap::default_workbench().with_overrides(editor_keymap_overrides(&settings));
    assert_eq!(
        user_keymap
            .chord_for_command("file.project.open")
            .unwrap()
            .to_string(),
        "Ctrl+Shift+O"
    );

    settings
        .set(
            SettingsScope::Session,
            &key,
            SettingValue::KeymapOverrides(overrides([("file.project.open", Some("Alt+O"))])),
        )
        .unwrap();
    let session_keymap =
        EditorKeymap::default_workbench().with_overrides(editor_keymap_overrides(&settings));
    assert_eq!(
        session_keymap
            .chord_for_command("file.project.open")
            .unwrap()
            .to_string(),
        "Alt+O"
    );
}

#[test]
fn keymap_reports_conflicts_after_a_settings_override() {
    let effective = EditorKeymap::default_workbench()
        .with_overrides(&overrides([("file.project.open", Some("Ctrl+S"))]));

    assert_eq!(
        effective.resolve(&EditorKeyChord::from_str("Ctrl+S").unwrap()),
        Some("file.project.open"),
        "candidate order must remain the operation-path order used before indexing"
    );
    let conflicts = effective.conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].chord().to_string(), "Ctrl+S");
    assert_eq!(
        conflicts[0]
            .command_ids()
            .iter()
            .map(|command_id| command_id.as_str())
            .collect::<Vec<_>>(),
        vec!["file.project.open", "file.project.save"]
    );
}

#[test]
fn keymap_overrides_round_trip_through_the_current_settings_shell() {
    let root = unique_temp_root("keymap-settings-roundtrip");
    let store = SettingsStore::from_roots(&root, None);
    let key = SettingsKey::parse(EDITOR_KEYMAP_OVERRIDES_KEY).unwrap();
    let expected = overrides([
        ("file.project.open", Some("Ctrl+Shift+O")),
        ("file.project.save", None),
    ]);
    let mut source = settings_registry_with_defaults();
    source
        .set(
            SettingsScope::User,
            &key,
            SettingValue::KeymapOverrides(expected.clone()),
        )
        .unwrap();
    store.save_from(SettingsScope::User, &source).unwrap();

    let encoded = fs::read_to_string(store.paths().user()).unwrap();
    assert!(encoded.contains("\"schema_id\": \"zircon.editor.settings\""));
    assert!(encoded.contains(EDITOR_KEYMAP_OVERRIDES_KEY));
    assert!(!encoded.contains("zircon.editor.keymap-user-layer"));

    let mut restored = settings_registry_with_defaults();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut restored).unwrap(),
        SettingsLoad::Loaded { .. }
    ));
    assert_eq!(
        editor_keymap_overrides(&restored),
        &expected,
        "the User layer must preserve the typed keymap delta"
    );
    assert_eq!(
        EditorKeymap::default_workbench()
            .with_overrides(editor_keymap_overrides(&restored))
            .chord_for_command("file.project.save"),
        None
    );
    remove_temp_root(&root);
}

#[test]
fn settings_store_rejects_the_retired_private_keymap_document() {
    let root = unique_temp_root("retired-keymap-document");
    let store = SettingsStore::from_roots(&root, None);
    fs::create_dir_all(&root).unwrap();
    fs::write(
        store.paths().user(),
        r#"{"bindings":{"file.project.open":"Ctrl+Shift+O"}}"#,
    )
    .unwrap();

    let mut settings = settings_registry_with_defaults();
    assert!(matches!(
        store.load_into(SettingsScope::User, &mut settings),
        Err(SettingsStoreError::Decode {
            source: SettingsDecodeError::LegacyPayload,
            ..
        })
    ));
    remove_temp_root(&root);
}

#[test]
fn keyboard_input_uses_the_generated_signature_index_for_large_keymaps() {
    let overrides = EditorKeymapOverrides::new(
        (0..10_000)
            .map(|index| {
                let command_id =
                    EditorOperationPath::parse(format!("benchmark.command.{index}")).unwrap();
                let chord = EditorKeyChord::from_str(&format!("Ctrl+F{}", index + 1)).unwrap();
                (command_id, Some(chord))
            })
            .collect(),
    );
    let keymap = EditorKeymap::default_workbench().with_overrides(&overrides);

    assert_eq!(keymap.signature_index.len(), 10_009);
    assert_eq!(
        keymap.resolve_keyboard_input(&keyboard_event("F10000", 0, true, false, false, false)),
        Some("benchmark.command.9999")
    );
}

#[test]
fn keyboard_signature_lookup_preserves_fallback_and_ignored_key_behavior() {
    let keymap = EditorKeymap::default_workbench();

    assert_eq!(
        keymap.resolve_keyboard_input(&keyboard_event("p", 80, true, true, false, false)),
        Some("editor.command.palette")
    );
    assert_eq!(
        keymap.resolve_keyboard_input(&keyboard_event("", 116, false, false, false, false)),
        Some("runtime.play_mode.enter")
    );
    assert_eq!(
        keymap.resolve_keyboard_input(&keyboard_event(
            "unidentified",
            46,
            false,
            false,
            false,
            false
        )),
        Some("scene.node.delete_selected")
    );
    assert_eq!(
        keymap.resolve_keyboard_input(&keyboard_event("DeadAcute", 0, false, false, false, false)),
        None
    );
    assert_eq!(
        keymap.resolve_keyboard_input(&keyboard_event("control", 17, false, false, false, false)),
        None
    );
    let mut released = keyboard_event("p", 80, true, true, false, false);
    released.state = UiKeyboardInputState::Released;
    assert_eq!(keymap.resolve_keyboard_input(&released), None);
}

#[test]
fn keyboard_hot_path_keeps_lookup_in_the_borrowed_signature_index() {
    let source = include_str!("../keymap.rs");

    assert!(source.contains("signature_index"));
    assert!(source.contains("EditorKeyboardChordInput::from_keyboard_input(keyboard)?"));
    assert!(!source.contains("EditorKeyChord::from_keyboard_input(keyboard)"));
    assert!(!source.contains(".iter()\n            .find(|binding| &binding.chord"));
}

#[test]
fn keyboard_signature_lookup_handles_a_million_event_storm_without_rebuilding_the_index() {
    let keymap = EditorKeymap::default_workbench();
    let event = keyboard_event("p", 80, true, true, false, false);
    let signature_count = keymap.signature_index.len();

    for _ in 0..1_000_000 {
        assert_eq!(
            std::hint::black_box(keymap.resolve_keyboard_input(&event)),
            Some("editor.command.palette")
        );
    }

    assert_eq!(keymap.signature_index.len(), signature_count);
}

fn keyboard_event(
    logical_key: &str,
    key_code: u32,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
) -> UiKeyboardInputEvent {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(1), UiInputSequence::new(1));
    metadata.modifiers = UiInputModifiers {
        control: ctrl,
        shift,
        alt,
        super_key: meta,
        ..UiInputModifiers::default()
    };
    UiKeyboardInputEvent {
        metadata,
        state: UiKeyboardInputState::Pressed,
        key_code,
        scan_code: None,
        physical_key: logical_key.to_string(),
        logical_key: logical_key.to_string(),
        text: None,
    }
}

fn overrides<const N: usize>(bindings: [(&str, Option<&str>); N]) -> EditorKeymapOverrides {
    EditorKeymapOverrides::new(
        bindings
            .into_iter()
            .map(|(command_id, chord)| {
                (
                    EditorOperationPath::parse(command_id).unwrap(),
                    chord.map(EditorKeyChord::from_str).transpose().unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
}

fn unique_temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon-editor-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn remove_temp_root(path: &Path) {
    let _ = fs::remove_dir_all(path);
}
