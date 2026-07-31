use woc_client::{
    setting_application_route, AudioSettingApplication, GamepadSettingApplication,
    HudSettingApplication, InputSettingApplication, RendererSettingApplication, SettingApplication,
    TouchSettingApplication, UiEffectsApplication, BOOL_SETTINGS, BOOL_SETTING_APPLICATIONS,
    NUMERIC_SETTINGS, NUMERIC_SETTING_APPLICATIONS,
};

#[test]
fn every_registered_setting_has_one_route_in_registry_order() {
    assert_eq!(
        NUMERIC_SETTING_APPLICATIONS.map(|route| route.key),
        NUMERIC_SETTINGS.map(|setting| setting.id)
    );
    assert_eq!(
        BOOL_SETTING_APPLICATIONS.map(|route| route.key),
        BOOL_SETTINGS.map(|setting| setting.id)
    );
}

#[test]
fn numeric_immediate_application_set_matches_the_target_switch() {
    let actual = NUMERIC_SETTING_APPLICATIONS
        .iter()
        .filter(|route| !route.applications.is_empty())
        .map(|route| route.key)
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            "cameraSpeed",
            "sfxVolume",
            "musicVolume",
            "voiceVolume",
            "brightness",
            "graphicsPreset",
            "browserEffects",
            "effectsQuality",
            "cameraFov",
            "renderScale",
            "fullscreen",
            "clickToMove",
            "clickToMoveButton",
            "interfaceMode",
            "touchLookSpeed",
            "touchOpacity",
            "weather",
            "joystickScale",
            "actionButtonScale",
            "joystickDeadzone",
            "gamepadStickDeadzone",
            "gamepadCameraSpeed",
            "gamepadVibration",
            "tooltipScale",
            "chatFontScale",
            "chatOpacity",
            "fctScale",
            "hudOpacity",
            "uiScale",
            "playerFrameScale",
            "targetFrameScale",
        ]
    );
}

#[test]
fn boolean_immediate_application_set_matches_the_target_switch() {
    let actual = BOOL_SETTING_APPLICATIONS
        .iter()
        .filter(|route| !route.applications.is_empty())
        .map(|route| route.key)
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            "mouseCamera",
            "lockCursorOnRotate",
            "gamepadEnabled",
            "gamepadInvertY",
            "leftHandedTouch",
            "mobileCameraJoystick",
            "attackMove",
            "touchInvertLook",
            "groundReticle",
            "aurasOnPlayerFrame",
            "reduceMotion",
            "highContrastText",
            "frostedPanels",
            "compactChat",
            "showFps",
            "showWalletOnCharacterScreen",
            "showDevBadges",
            "showOwnNameplate",
            "invertLookY",
            "voiceEnabled",
            "footstepSfx",
            "landingHighContrast",
            "showSecondaryActionBar",
            "showDailyRewardsChest",
        ]
    );
}

#[test]
fn multi_owner_routes_preserve_target_side_effect_order() {
    assert_eq!(
        setting_application_route("sfxVolume")
            .expect("sfx route")
            .applications,
        &[
            SettingApplication::Audio(AudioSettingApplication::AmbientVolume),
            SettingApplication::Audio(AudioSettingApplication::SoundEffectsVolume),
        ]
    );
    assert_eq!(
        setting_application_route("uiScale")
            .expect("ui scale route")
            .applications,
        &[
            SettingApplication::RootCssVariable("--ui-scale"),
            SettingApplication::Hud(HudSettingApplication::ReapplySavedGeometry),
        ]
    );
    assert_eq!(
        setting_application_route("mobileCameraJoystick")
            .expect("camera joystick route")
            .applications,
        &[
            SettingApplication::BodyClass("mobile-camera-joystick-on"),
            SettingApplication::Touch(TouchSettingApplication::CameraJoystickEnabled),
        ]
    );
    assert_eq!(
        setting_application_route("reduceMotion")
            .expect("reduce motion route")
            .applications,
        &[
            SettingApplication::BodyClass("reduce-motion"),
            SettingApplication::UiEffects(UiEffectsApplication::ApplyNow),
        ]
    );
}

#[test]
fn subsystem_routes_remain_typed_instead_of_string_dispatched() {
    assert_eq!(
        setting_application_route("cameraSpeed")
            .expect("camera route")
            .applications,
        &[SettingApplication::Input(
            InputSettingApplication::CameraSpeed
        )]
    );
    assert_eq!(
        setting_application_route("brightness")
            .expect("brightness route")
            .applications,
        &[SettingApplication::Renderer(
            RendererSettingApplication::Brightness
        )]
    );
    assert_eq!(
        setting_application_route("gamepadVibration")
            .expect("vibration route")
            .applications,
        &[SettingApplication::Gamepad(
            GamepadSettingApplication::Vibration
        )]
    );
}

#[test]
fn persist_only_and_unknown_settings_are_distinct() {
    for key in [
        "filterProfanity",
        "startAttackOnAbilityUse",
        "showAttackButton",
        "partyFrameStyle",
        "graphicsDefaultApplied",
    ] {
        assert!(setting_application_route(key)
            .expect("known persisted setting")
            .applications
            .is_empty());
    }
    assert!(setting_application_route("notASetting").is_none());
}
