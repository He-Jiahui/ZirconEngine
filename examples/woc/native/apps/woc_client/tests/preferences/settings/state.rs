use woc_client::{
    click_move_button_label, normalize_click_move_button, ClientSettings, BOOL_SETTINGS,
    NUMERIC_SETTINGS,
};

#[test]
fn defaults_cover_every_registered_setting() {
    let settings = ClientSettings::default();
    assert_eq!(settings.numeric_len(), NUMERIC_SETTINGS.len());
    assert_eq!(settings.boolean_len(), BOOL_SETTINGS.len());
    for setting in NUMERIC_SETTINGS {
        assert_eq!(
            settings.numeric(setting.id),
            Some(setting.default),
            "{}",
            setting.id
        );
    }
    for setting in BOOL_SETTINGS {
        assert_eq!(
            settings.boolean(setting.id),
            Some(setting.default),
            "{}",
            setting.id
        );
    }
}

#[test]
fn numeric_updates_clamp_and_non_finite_values_restore_the_default() {
    let mut settings = ClientSettings::default();
    assert_eq!(settings.set_numeric("cameraSpeed", 99.0), Some(1.25));
    assert_eq!(settings.set_numeric("cameraSpeed", -5.0), Some(0.25));
    assert_eq!(settings.set_numeric("cameraFov", 75.0), Some(75.0));
    assert_eq!(settings.set_numeric("joystickDeadzone", 0.0), Some(0.1));
    assert_eq!(settings.set_numeric("brightness", f64::NAN), Some(1.0));
    assert_eq!(
        settings.set_numeric("gamepadCameraSpeed", f64::INFINITY),
        Some(2.4)
    );
}

#[test]
fn boolean_updates_cover_gamepad_touch_and_interface_preferences() {
    let mut settings = ClientSettings::default();
    for id in [
        "gamepadInvertY",
        "leftHandedTouch",
        "mobileCameraJoystick",
        "reduceMotion",
        "showFps",
    ] {
        assert_eq!(settings.set_boolean(id, true), Some(true));
        assert_eq!(settings.boolean(id), Some(true));
    }
    assert_eq!(settings.set_boolean("showOwnNameplate", false), Some(false));
}

#[test]
fn unknown_updates_are_atomic() {
    let mut settings = ClientSettings::default();
    let before = settings.clone();
    assert_eq!(settings.set_numeric("missing", 1.0), None);
    assert_eq!(settings.set_boolean("missing", true), None);
    assert_eq!(settings.numeric("missing"), None);
    assert_eq!(settings.boolean("missing"), None);
    assert_eq!(settings, before);
}

#[test]
fn reset_restores_all_eighty_four_defaults() {
    let mut settings = ClientSettings::default();
    for setting in NUMERIC_SETTINGS {
        settings.set_numeric(setting.id, setting.max);
    }
    for setting in BOOL_SETTINGS {
        settings.set_boolean(setting.id, !setting.default);
    }
    settings.reset();
    for setting in NUMERIC_SETTINGS {
        assert_eq!(settings.numeric(setting.id), Some(setting.default));
    }
    for setting in BOOL_SETTINGS {
        assert_eq!(settings.boolean(setting.id), Some(setting.default));
    }
}

#[test]
fn all_returns_an_independent_snapshot() {
    let settings = ClientSettings::default();
    let mut snapshot = settings.all();
    snapshot.set_numeric("cameraSpeed", 1.25);
    snapshot.set_boolean("showFps", true);
    assert_eq!(settings.numeric("cameraSpeed"), Some(0.7));
    assert_eq!(settings.boolean("showFps"), Some(false));
}

#[test]
fn click_to_move_button_normalization_matches_target_labels() {
    assert_eq!(normalize_click_move_button(0.0), 0);
    assert_eq!(normalize_click_move_button(0.4), 0);
    assert_eq!(normalize_click_move_button(1.0), 2);
    assert_eq!(normalize_click_move_button(2.0), 2);
    assert_eq!(click_move_button_label(0.0), "Left Click");
    assert_eq!(click_move_button_label(2.0), "Right Click");
}
