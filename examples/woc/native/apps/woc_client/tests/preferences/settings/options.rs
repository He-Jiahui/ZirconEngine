use woc_client::{
    bool_toggle_next, build_audio_controls, build_controller_controls, build_graphics_controls,
    build_interface_controls, build_options_menu, numeric_toggle_is_on, numeric_toggle_next,
    ClientSettings, OptionsEnvironment, OptionsMenuAction, OptionsPanelId, SettingsControl,
    SliderFormat,
};

fn keys(controls: &[SettingsControl]) -> Vec<String> {
    controls
        .iter()
        .map(|control| match control {
            SettingsControl::Note { text_key } => format!("note:{text_key}"),
            SettingsControl::MusicToggle { .. } => "musicToggle".to_string(),
            _ => control.key().expect("setting control key").to_string(),
        })
        .collect()
}

fn control<'a>(controls: &'a [SettingsControl], key: &str) -> &'a SettingsControl {
    controls
        .iter()
        .find(|control| control.key() == Some(key))
        .expect("control must exist")
}

#[test]
fn primitive_toggle_dispatch_matches_the_target_thresholds() {
    assert_eq!(numeric_toggle_next(0.0), 1.0);
    assert_eq!(numeric_toggle_next(0.4), 1.0);
    assert_eq!(numeric_toggle_next(0.5), 0.0);
    assert_eq!(numeric_toggle_next(1.0), 0.0);
    assert!(!numeric_toggle_is_on(0.49));
    assert!(numeric_toggle_is_on(0.5));
    assert!(!bool_toggle_next(true));
    assert!(bool_toggle_next(false));
}

#[test]
fn desktop_graphics_controls_preserve_base_order_and_live_ranges() {
    let mut settings = ClientSettings::default();
    settings.set_numeric("graphicsPreset", 4.0);
    settings.set_numeric("cameraSpeed", 0.9);
    settings.set_numeric("cameraFov", 75.0);

    let controls = build_graphics_controls(
        &settings,
        OptionsEnvironment {
            touch: false,
            native_shell: false,
        },
    );

    assert_eq!(
        keys(&controls),
        [
            "graphicsPreset",
            "browserEffects",
            "note:hudChrome.options.browserEffectsNote",
            "interfaceMode",
            "note:hudChrome.options.interfaceModeNote",
            "cameraSpeed",
            "brightness",
            "cameraFov",
            "renderScale",
            "fullscreen",
            "showOverflowXp",
            "weather",
        ]
    );
    let SettingsControl::Slider(camera) = control(&controls, "cameraSpeed") else {
        panic!("cameraSpeed must be a slider");
    };
    assert_eq!(
        (camera.min, camera.max, camera.step, camera.value),
        (0.25, 1.25, 0.05, 0.9)
    );
    assert_eq!(camera.format, SliderFormat::Percent);
    let SettingsControl::Slider(fov) = control(&controls, "cameraFov") else {
        panic!("cameraFov must be a slider");
    };
    assert_eq!(
        (fov.step, fov.value, fov.format),
        (1.0, 75.0, SliderFormat::Degrees)
    );
}

#[test]
fn advanced_graphics_preset_exposes_four_low_high_choices() {
    let mut settings = ClientSettings::default();
    settings.set_numeric("graphicsPreset", 5.0);

    let controls = build_graphics_controls(
        &settings,
        OptionsEnvironment {
            touch: false,
            native_shell: false,
        },
    );

    assert_eq!(
        &keys(&controls)[..5],
        [
            "graphicsPreset",
            "terrainDetail",
            "foliageDensity",
            "effectsQuality",
            "shadowQuality",
        ]
    );
    let SettingsControl::Choice(terrain) = control(&controls, "terrainDetail") else {
        panic!("terrainDetail must be a choice");
    };
    assert_eq!(
        terrain
            .options
            .iter()
            .map(|option| option.value)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(!terrain.rerender);
}

#[test]
fn native_and_touch_graphics_gates_match_the_target_shell() {
    let settings = ClientSettings::default();
    let controls = build_graphics_controls(
        &settings,
        OptionsEnvironment {
            touch: true,
            native_shell: true,
        },
    );
    let all_keys = keys(&controls);

    assert!(!all_keys.iter().any(|key| key == "interfaceMode"));
    let SettingsControl::Choice(preset) = control(&controls, "graphicsPreset") else {
        panic!("graphicsPreset must be a choice");
    };
    assert_eq!(
        preset
            .options
            .iter()
            .map(|option| option.value)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    for touch_key in [
        "touchLookSpeed",
        "touchOpacity",
        "joystickScale",
        "actionButtonScale",
        "joystickDeadzone",
        "touchInvertLook",
        "mobileCameraJoystick",
        "leftHandedTouch",
    ] {
        assert!(
            all_keys.iter().any(|key| key == touch_key),
            "missing {touch_key}"
        );
    }
}

#[test]
fn audio_controls_preserve_three_sliders_music_owner_and_boolean_rows() {
    assert_eq!(
        keys(&build_audio_controls(&ClientSettings::default())),
        [
            "sfxVolume",
            "musicVolume",
            "voiceVolume",
            "musicToggle",
            "voiceEnabled",
            "footstepSfx",
            "interfaceSfx",
            "clickFeedback",
        ]
    );
}

#[test]
fn controller_controls_preserve_toggle_then_slider_order() {
    let controls = build_controller_controls(&ClientSettings::default());
    assert_eq!(
        keys(&controls),
        [
            "gamepadEnabled",
            "gamepadInvertY",
            "gamepadStickDeadzone",
            "gamepadCameraSpeed",
            "gamepadVibration",
        ]
    );
    let SettingsControl::Slider(camera) = control(&controls, "gamepadCameraSpeed") else {
        panic!("gamepadCameraSpeed must be a slider");
    };
    assert_eq!(camera.format, SliderFormat::OneDecimal);
}

#[test]
fn interface_controls_match_the_exact_target_row_order_including_duplicate_attack_button() {
    let controls = build_interface_controls(&ClientSettings::default());
    assert_eq!(
        keys(&controls),
        [
            "uiScale",
            "playerFrameScale",
            "targetFrameScale",
            "note:hudChrome.partyFrames.section",
            "partyFrameStyle",
            "partyFrameScale",
            "partyFrameWidth",
            "partyFrameHeight",
            "partyFrameSpacing",
            "partyFrameColumns",
            "partyFrameHealthText",
            "partyFrameSort",
            "partyFrameShowResource",
            "partyFrameShowAbsorbs",
            "partyFrameShowAuras",
            "partyFrameShowSelf",
            "hudOpacity",
            "tooltipScale",
            "fctScale",
            "chatFontScale",
            "chatOpacity",
            "compactChat",
            "frostedPanels",
            "highContrastText",
            "reduceMotion",
            "showWalletOnCharacterScreen",
            "showWalletOnPlayerCard",
            "showDevBadges",
            "showOwnNameplate",
            "landingHighContrast",
            "invertLookY",
            "startAttackOnAbilityUse",
            "showAttackButton",
            "walkByAutoloot",
            "groundReticle",
            "mouseoverCast",
            "aurasOnPlayerFrame",
            "showItemLevel",
            "showSecondaryActionBar",
            "showTargetOfTarget",
            "showAttackButton",
            "showDailyRewardsChest",
        ]
    );
    let SettingsControl::Slider(ui_scale) = control(&controls, "uiScale") else {
        panic!("uiScale must be a slider");
    };
    assert!(ui_scale.commit_on_change);
    let SettingsControl::Slider(tooltip_scale) = control(&controls, "tooltipScale") else {
        panic!("tooltipScale must be a slider");
    };
    assert!(!tooltip_scale.commit_on_change);
}

#[test]
fn options_menu_routes_exact_panels_and_only_adds_bug_report_when_available() {
    let offline = build_options_menu(false);
    assert_eq!(offline.len(), 8);
    assert_eq!(
        offline[0].action,
        OptionsMenuAction::GoTo(OptionsPanelId::Keybinds)
    );
    assert_eq!(
        offline[5].action,
        OptionsMenuAction::GoTo(OptionsPanelId::Performance)
    );
    assert_eq!(offline[6].action, OptionsMenuAction::Logout);
    assert_eq!(offline[7].action, OptionsMenuAction::Close);

    let online = build_options_menu(true);
    assert_eq!(online.len(), 9);
    assert_eq!(
        online[6].action,
        OptionsMenuAction::GoTo(OptionsPanelId::BugReport)
    );
    assert_eq!(online[7].action, OptionsMenuAction::Logout);
    assert_eq!(online[8].action, OptionsMenuAction::Close);
}

#[test]
fn identical_settings_and_environment_produce_identical_control_trees() {
    let settings = ClientSettings::default();
    let environment = OptionsEnvironment {
        touch: true,
        native_shell: false,
    };
    assert_eq!(
        build_graphics_controls(&settings, environment),
        build_graphics_controls(&settings, environment)
    );
    assert_eq!(
        build_interface_controls(&settings),
        build_interface_controls(&settings)
    );
}
