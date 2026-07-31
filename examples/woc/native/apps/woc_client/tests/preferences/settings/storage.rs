use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};

use serde_json::Value;
use woc_client::GraphicsRuntimeHints;
use woc_client::{
    setting_application_route, ClientSettingValue, GamepadSettingApplication, PreferenceStorage,
    RendererSettingApplication, SettingApplication, StoredClientSettings, BOOL_SETTINGS,
    CLIENT_SETTINGS_STORAGE_KEY, NUMERIC_SETTINGS,
};

#[derive(Clone, Default)]
struct MemoryStorage {
    values: Rc<RefCell<BTreeMap<String, String>>>,
    fail_reads: Rc<Cell<bool>>,
    fail_writes: Rc<Cell<bool>>,
}

impl MemoryStorage {
    fn insert(&self, key: &str, value: &str) {
        self.values
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
    }

    fn get(&self, key: &str) -> Option<String> {
        self.values.borrow().get(key).cloned()
    }
}

impl PreferenceStorage for MemoryStorage {
    type Error = ();

    fn read(&self, key: &str) -> Result<Option<String>, Self::Error> {
        if self.fail_reads.get() {
            Err(())
        } else {
            Ok(self.get(key))
        }
    }

    fn write(&self, key: &str, value: &str) -> Result<(), Self::Error> {
        if self.fail_writes.get() {
            Err(())
        } else {
            self.insert(key, value);
            Ok(())
        }
    }
}

#[test]
fn fresh_settings_use_defaults_without_writing_storage() {
    let storage = MemoryStorage::default();
    let settings = StoredClientSettings::new(storage.clone());
    assert_eq!(settings.numeric("graphicsPreset"), Some(2.0));
    assert_eq!(settings.boolean("graphicsDefaultApplied"), Some(false));
    assert!(storage.get(CLIENT_SETTINGS_STORAGE_KEY).is_none());
}

#[test]
fn numeric_and_boolean_changes_round_trip_across_instances() {
    let storage = MemoryStorage::default();
    let mut settings = StoredClientSettings::new(storage.clone());
    assert_eq!(settings.set_numeric("cameraSpeed", 0.4), Some(0.4));
    assert_eq!(settings.set_numeric("musicVolume", 0.2), Some(0.2));
    assert_eq!(settings.set_boolean("mouseCamera", true), Some(true));

    let reloaded = StoredClientSettings::new(storage);
    assert_eq!(reloaded.numeric("cameraSpeed"), Some(0.4));
    assert_eq!(reloaded.numeric("musicVolume"), Some(0.2));
    assert_eq!(reloaded.boolean("mouseCamera"), Some(true));
}

#[test]
fn missing_or_wrong_typed_keys_fall_back_and_numeric_values_clamp() {
    let storage = MemoryStorage::default();
    storage.insert(
        CLIENT_SETTINGS_STORAGE_KEY,
        r#"{
            "cameraSpeed": 99,
            "brightness": "bright",
            "showFps": true,
            "gamepadEnabled": 0,
            "gamepadCameraSpeed": 1e400
        }"#,
    );
    let settings = StoredClientSettings::new(storage);
    assert_eq!(settings.numeric("cameraSpeed"), Some(1.25));
    assert_eq!(settings.numeric("brightness"), Some(1.0));
    assert_eq!(settings.numeric("gamepadCameraSpeed"), Some(2.4));
    assert_eq!(settings.numeric("fullscreen"), Some(1.0));
    assert_eq!(settings.boolean("showFps"), Some(true));
    assert_eq!(settings.boolean("gamepadEnabled"), Some(true));
}

#[test]
fn corrupt_null_scalar_or_array_storage_uses_defaults() {
    for raw in ["{invalid", "null", "42", "false", r#"["garbage"]"#] {
        let storage = MemoryStorage::default();
        storage.insert(CLIENT_SETTINGS_STORAGE_KEY, raw);
        let settings = StoredClientSettings::new(storage);
        assert_eq!(settings.numeric("cameraSpeed"), Some(0.7), "{raw}");
        assert_eq!(settings.boolean("gamepadEnabled"), Some(true), "{raw}");
    }
}

#[test]
fn every_successful_change_saves_all_eighty_four_known_settings() {
    let storage = MemoryStorage::default();
    let mut settings = StoredClientSettings::new(storage.clone());
    assert_eq!(settings.set_boolean("showFps", true), Some(true));

    let raw = storage
        .get(CLIENT_SETTINGS_STORAGE_KEY)
        .expect("settings JSON");
    let encoded: Value = serde_json::from_str(&raw).expect("stored JSON object");
    assert_eq!(
        encoded.as_object().map(|object| object.len()),
        Some(NUMERIC_SETTINGS.len() + BOOL_SETTINGS.len())
    );
    assert_eq!(encoded["showFps"], true);
    assert_eq!(encoded["graphicsDefaultApplied"], false);
}

#[test]
fn graphics_default_marker_changes_only_when_explicitly_set_and_reset_clears_it() {
    let storage = MemoryStorage::default();
    let mut settings = StoredClientSettings::new(storage.clone());
    settings.set_boolean("showFps", true);
    assert_eq!(
        StoredClientSettings::new(storage.clone()).boolean("graphicsDefaultApplied"),
        Some(false)
    );
    settings.set_boolean("graphicsDefaultApplied", true);
    assert_eq!(
        StoredClientSettings::new(storage.clone()).boolean("graphicsDefaultApplied"),
        Some(true)
    );
    settings.reset();
    assert_eq!(
        StoredClientSettings::new(storage).boolean("graphicsDefaultApplied"),
        Some(false)
    );
}

#[test]
fn reset_persists_every_default() {
    let storage = MemoryStorage::default();
    let mut settings = StoredClientSettings::new(storage.clone());
    settings.set_numeric("cameraSpeed", 1.2);
    settings.set_boolean("mobileCameraJoystick", true);
    settings.reset();

    let reloaded = StoredClientSettings::new(storage);
    assert_eq!(reloaded.numeric("cameraSpeed"), Some(0.7));
    assert_eq!(reloaded.boolean("mobileCameraJoystick"), Some(false));
}

#[test]
fn unavailable_storage_degrades_to_defaults_and_keeps_session_changes() {
    let storage = MemoryStorage::default();
    storage.fail_reads.set(true);
    storage.fail_writes.set(true);
    let mut settings = StoredClientSettings::new(storage.clone());
    assert_eq!(settings.numeric("cameraSpeed"), Some(0.7));
    assert_eq!(settings.set_numeric("cameraSpeed", 0.4), Some(0.4));
    assert_eq!(settings.set_boolean("showFps", true), Some(true));
    assert_eq!(settings.numeric("cameraSpeed"), Some(0.4));
    assert_eq!(settings.boolean("showFps"), Some(true));
    assert!(storage.get(CLIENT_SETTINGS_STORAGE_KEY).is_none());
}

#[test]
fn committed_changes_return_normalized_values_and_typed_application_routes() {
    let storage = MemoryStorage::default();
    let mut settings = StoredClientSettings::new(storage.clone());

    let render_scale = settings
        .set_numeric_with_application("renderScale", 4.0)
        .expect("known numeric setting");
    assert_eq!(render_scale.key, "renderScale");
    assert_eq!(render_scale.value, ClientSettingValue::Numeric(1.0));
    assert_eq!(
        render_scale.applications,
        &[SettingApplication::Renderer(
            RendererSettingApplication::RenderScale
        )]
    );

    let gamepad = settings
        .set_boolean_with_application("gamepadEnabled", false)
        .expect("known boolean setting");
    assert_eq!(gamepad.key, "gamepadEnabled");
    assert_eq!(gamepad.value, ClientSettingValue::Boolean(false));
    assert_eq!(
        gamepad.applications,
        &[SettingApplication::Gamepad(
            GamepadSettingApplication::Enabled
        )]
    );

    let reloaded = StoredClientSettings::new(storage);
    assert_eq!(reloaded.numeric("renderScale"), Some(1.0));
    assert_eq!(reloaded.boolean("gamepadEnabled"), Some(false));
}

#[test]
fn startup_application_plan_covers_all_settings_in_target_order() {
    let settings = StoredClientSettings::new(MemoryStorage::default());
    let plan = settings.application_plan();
    let expected_keys = NUMERIC_SETTINGS
        .iter()
        .map(|setting| setting.id)
        .chain(BOOL_SETTINGS.iter().map(|setting| setting.id))
        .collect::<Vec<_>>();

    assert_eq!(plan.len(), 84);
    assert_eq!(
        plan.iter().map(|change| change.key).collect::<Vec<_>>(),
        expected_keys
    );
    assert_eq!(plan[0].value, ClientSettingValue::Numeric(0.7));
    assert_eq!(plan[42].value, ClientSettingValue::Numeric(0.0));
    assert_eq!(plan[43].value, ClientSettingValue::Boolean(false));
    assert_eq!(plan[83].value, ClientSettingValue::Boolean(false));
    for change in plan {
        assert_eq!(
            change.applications,
            setting_application_route(change.key)
                .expect("registered route")
                .applications
        );
    }
}

#[test]
fn unknown_changes_return_no_application_and_do_not_write() {
    let storage = MemoryStorage::default();
    let mut settings = StoredClientSettings::new(storage.clone());

    assert!(settings
        .set_numeric_with_application("missing", 1.0)
        .is_none());
    assert!(settings
        .set_boolean_with_application("missing", true)
        .is_none());
    assert!(storage.get(CLIENT_SETTINGS_STORAGE_KEY).is_none());
}

#[test]
fn conclusive_first_run_graphics_default_is_persisted_and_marked_once() {
    let storage = MemoryStorage::default();
    let mut settings = StoredClientSettings::new(storage.clone());
    let hints = GraphicsRuntimeHints {
        gpu_renderer: Some("Google SwiftShader"),
        ..GraphicsRuntimeHints::default()
    };

    let changes = settings.initialize_graphics_preset(&hints, false);
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].key, "graphicsPreset");
    assert_eq!(changes[0].value, ClientSettingValue::Numeric(1.0));
    assert_eq!(changes[1].key, "graphicsDefaultApplied");
    assert_eq!(changes[1].value, ClientSettingValue::Boolean(true));
    assert_eq!(settings.numeric("graphicsPreset"), Some(1.0));
    assert_eq!(settings.boolean("graphicsDefaultApplied"), Some(true));
    assert!(settings
        .initialize_graphics_preset(&hints, false)
        .is_empty());

    let reloaded = StoredClientSettings::new(storage);
    assert_eq!(reloaded.numeric("graphicsPreset"), Some(1.0));
    assert_eq!(reloaded.boolean("graphicsDefaultApplied"), Some(true));
}

#[test]
fn inconclusive_first_run_keeps_medium_unmarked_for_a_later_probe() {
    let storage = MemoryStorage::default();
    let mut settings = StoredClientSettings::new(storage.clone());

    assert!(settings
        .initialize_graphics_preset(&GraphicsRuntimeHints::default(), false)
        .is_empty());
    assert_eq!(settings.numeric("graphicsPreset"), Some(2.0));
    assert_eq!(settings.boolean("graphicsDefaultApplied"), Some(false));
    assert!(storage.get(CLIENT_SETTINGS_STORAGE_KEY).is_none());
}

#[test]
fn native_startup_clamps_saved_ultra_or_advanced_to_high() {
    for preset in [4, 5] {
        let storage = MemoryStorage::default();
        storage.insert(
            CLIENT_SETTINGS_STORAGE_KEY,
            &format!(r#"{{"graphicsPreset":{preset},"graphicsDefaultApplied":true}}"#),
        );
        let mut settings = StoredClientSettings::new(storage.clone());

        let changes = settings.initialize_graphics_preset(&GraphicsRuntimeHints::default(), true);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].key, "graphicsPreset");
        assert_eq!(changes[0].value, ClientSettingValue::Numeric(3.0));
        assert_eq!(settings.numeric("graphicsPreset"), Some(3.0));
        assert_eq!(
            StoredClientSettings::new(storage).numeric("graphicsPreset"),
            Some(3.0)
        );
    }
}
