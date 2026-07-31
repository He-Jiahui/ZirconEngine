use std::collections::BTreeSet;

use woc_client::{bool_setting, numeric_setting, BOOL_SETTINGS, NUMERIC_SETTINGS};

#[test]
fn numeric_registry_matches_all_forty_three_target_ids_in_order() {
    assert_eq!(
        NUMERIC_SETTINGS
            .iter()
            .map(|setting| setting.id)
            .collect::<Vec<_>>(),
        [
            "cameraSpeed",
            "sfxVolume",
            "musicVolume",
            "voiceVolume",
            "brightness",
            "graphicsPreset",
            "browserEffects",
            "terrainDetail",
            "foliageDensity",
            "effectsQuality",
            "shadowQuality",
            "cameraFov",
            "renderScale",
            "fullscreen",
            "showOverflowXp",
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
            "partyFrameStyle",
            "partyFrameScale",
            "partyFrameWidth",
            "partyFrameHeight",
            "partyFrameSpacing",
            "partyFrameColumns",
            "partyFrameHealthText",
            "partyFrameSort",
        ]
    );
}

#[test]
fn boolean_registry_matches_all_forty_one_target_ids_in_order() {
    assert_eq!(
        BOOL_SETTINGS
            .iter()
            .map(|setting| setting.id)
            .collect::<Vec<_>>(),
        [
            "mouseCamera",
            "lockCursorOnRotate",
            "gamepadEnabled",
            "gamepadInvertY",
            "leftHandedTouch",
            "mobileCameraJoystick",
            "filterProfanity",
            "attackMove",
            "touchInvertLook",
            "startAttackOnAbilityUse",
            "showAttackButton",
            "walkByAutoloot",
            "groundReticle",
            "aurasOnPlayerFrame",
            "mouseoverCast",
            "partyFrameShowResource",
            "partyFrameShowAbsorbs",
            "partyFrameShowAuras",
            "partyFrameShowSelf",
            "reduceMotion",
            "highContrastText",
            "frostedPanels",
            "compactChat",
            "showFps",
            "showWalletOnCharacterScreen",
            "showWalletOnPlayerCard",
            "showDevBadges",
            "showOwnNameplate",
            "invertLookY",
            "voiceEnabled",
            "footstepSfx",
            "interfaceSfx",
            "clickFeedback",
            "landingHighContrast",
            "questTrackerCollapsed",
            "deedTrackerCollapsed",
            "showItemLevel",
            "showSecondaryActionBar",
            "showTargetOfTarget",
            "showDailyRewardsChest",
            "graphicsDefaultApplied",
        ]
    );
}

#[test]
fn registries_are_unique_well_formed_and_pin_high_risk_defaults() {
    assert_eq!(
        NUMERIC_SETTINGS
            .iter()
            .map(|setting| setting.id)
            .collect::<BTreeSet<_>>()
            .len(),
        43
    );
    assert_eq!(
        BOOL_SETTINGS
            .iter()
            .map(|setting| setting.id)
            .collect::<BTreeSet<_>>()
            .len(),
        41
    );
    for setting in NUMERIC_SETTINGS {
        assert!(setting.min <= setting.default, "{}", setting.id);
        assert!(setting.default <= setting.max, "{}", setting.id);
    }
    assert_eq!(numeric_setting("cameraSpeed").unwrap().default, 0.7);
    assert_eq!(numeric_setting("graphicsPreset").unwrap().default, 2.0);
    assert_eq!(numeric_setting("joystickDeadzone").unwrap().default, 0.22);
    assert_eq!(
        numeric_setting("gamepadStickDeadzone").unwrap().default,
        0.18
    );
    assert_eq!(numeric_setting("gamepadCameraSpeed").unwrap().default, 2.4);
    assert_eq!(
        bool_setting("graphicsDefaultApplied").unwrap().default,
        false
    );
    assert_eq!(bool_setting("gamepadEnabled").unwrap().default, true);
    assert_eq!(bool_setting("attackMove").unwrap().default, false);
}
