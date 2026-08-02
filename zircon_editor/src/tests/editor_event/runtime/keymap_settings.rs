use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::commands::{EditorKeyChord, EditorKeymap};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::settings::{
    EDITOR_KEYMAP_OVERRIDES_KEY, EditorKeymapOverrides, SETTINGS_USER_ROOT_ENV, SettingValue,
    SettingsKey, SettingsScope, SettingsStore, settings_registry_with_defaults,
};
use crate::ui::host::module::EDITOR_KEYMAP_NAME;

use super::super::support::{EventRuntimeHarness, env_lock};

#[test]
fn host_and_manager_service_share_the_user_settings_keymap() {
    let _guard = env_lock();
    let root = temporary_settings_root("host-keymap-settings");
    write_user_keymap_override(&root);

    let previous_root = env::var_os(SETTINGS_USER_ROOT_ENV);
    env::set_var(SETTINGS_USER_ROOT_ENV, &root);
    let harness = EventRuntimeHarness::new("editor-host-keymap-settings");
    restore_environment(SETTINGS_USER_ROOT_ENV, previous_root);

    let manager_keymap = harness
        .core
        .resolve_manager::<EditorKeymap>(EDITOR_KEYMAP_NAME)
        .expect("EditorKeymap manager service should resolve");
    assert_eq!(harness.runtime.keymap(), manager_keymap.as_ref());
    assert_eq!(
        harness
            .runtime
            .keymap()
            .chord_for_command("file.project.open")
            .expect("the overridden command should remain bound")
            .to_string(),
        "Alt+O"
    );

    let _ = fs::remove_dir_all(root);
}

fn write_user_keymap_override(root: &Path) {
    let key = SettingsKey::parse(EDITOR_KEYMAP_OVERRIDES_KEY)
        .expect("the built-in keymap setting key should be valid");
    let mut settings = settings_registry_with_defaults();
    settings
        .set(
            SettingsScope::User,
            &key,
            SettingValue::KeymapOverrides(EditorKeymapOverrides::new(
                [(
                    EditorOperationPath::parse("file.project.open")
                        .expect("the built-in operation path should be valid"),
                    Some(
                        "Alt+O"
                            .parse::<EditorKeyChord>()
                            .expect("the test chord should be valid"),
                    ),
                )]
                .into_iter()
                .collect(),
            )),
        )
        .expect("the User layer should accept keymap overrides");
    SettingsStore::from_roots(root, None)
        .save_from(SettingsScope::User, &settings)
        .expect("the versioned settings store should save the User layer");
}

fn restore_environment(key: &str, previous: Option<std::ffi::OsString>) {
    match previous {
        Some(value) => env::set_var(key, value),
        None => env::remove_var(key),
    }
}

fn temporary_settings_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock should be after the epoch")
        .as_nanos();
    env::temp_dir().join(format!(
        "zircon-editor-{label}-{}-{nonce}",
        std::process::id()
    ))
}
