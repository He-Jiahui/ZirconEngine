use super::{bool_setting, numeric_setting, ClientSettings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliderFormat {
    Percent,
    Degrees,
    OneDecimal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderControl {
    pub key: &'static str,
    pub label_key: &'static str,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub value: f64,
    pub format: SliderFormat,
    pub commit_on_change: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToggleControl {
    pub key: &'static str,
    pub label_key: &'static str,
    pub on: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChoiceOption {
    pub value: i32,
    pub label_key: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChoiceControl {
    pub key: &'static str,
    pub label_key: &'static str,
    pub current: i32,
    pub options: Vec<ChoiceOption>,
    pub rerender: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SettingsControl {
    Slider(SliderControl),
    NumericToggle(ToggleControl),
    BoolToggle(ToggleControl),
    Choice(ChoiceControl),
    Note { text_key: &'static str },
    MusicToggle { label_key: &'static str },
}

impl SettingsControl {
    pub fn key(&self) -> Option<&'static str> {
        match self {
            Self::Slider(control) => Some(control.key),
            Self::NumericToggle(control) | Self::BoolToggle(control) => Some(control.key),
            Self::Choice(control) => Some(control.key),
            Self::Note { .. } | Self::MusicToggle { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OptionsEnvironment {
    pub touch: bool,
    pub native_shell: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionsPanelId {
    Keybinds,
    Controller,
    Graphics,
    Interface,
    Audio,
    Performance,
    BugReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptionsMenuAction {
    GoTo(OptionsPanelId),
    Logout,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionsMenuEntry {
    pub label_key: &'static str,
    pub action: OptionsMenuAction,
}

pub fn numeric_toggle_next(current: f64) -> f64 {
    if current >= 0.5 {
        0.0
    } else {
        1.0
    }
}

pub fn numeric_toggle_is_on(current: f64) -> bool {
    current >= 0.5
}

pub fn bool_toggle_next(current: bool) -> bool {
    !current
}

pub fn build_options_menu(bug_report_available: bool) -> Vec<OptionsMenuEntry> {
    let mut entries = vec![
        menu_entry("hud.options.keyBindings", OptionsPanelId::Keybinds),
        menu_entry("hudChrome.controller.title", OptionsPanelId::Controller),
        menu_entry("hud.options.graphics", OptionsPanelId::Graphics),
        menu_entry("hud.options.interface", OptionsPanelId::Interface),
        menu_entry("hud.options.audio", OptionsPanelId::Audio),
        menu_entry("hudChrome.perf.title", OptionsPanelId::Performance),
    ];
    if bug_report_available {
        entries.push(menu_entry(
            "hudChrome.bugReport.menuButton",
            OptionsPanelId::BugReport,
        ));
    }
    entries.push(OptionsMenuEntry {
        label_key: "hud.options.logout",
        action: OptionsMenuAction::Logout,
    });
    entries.push(OptionsMenuEntry {
        label_key: "hud.options.returnToGame",
        action: OptionsMenuAction::Close,
    });
    entries
}

pub fn build_graphics_controls(
    settings: &ClientSettings,
    environment: OptionsEnvironment,
) -> Vec<SettingsControl> {
    let mut controls = Vec::new();
    let mut preset_options = vec![
        option(1, "hud.options.graphicsPresetLow"),
        option(2, "hud.options.graphicsPresetMedium"),
        option(3, "hud.options.graphicsPresetHigh"),
    ];
    if !environment.native_shell {
        preset_options.extend([
            option(4, "hud.options.graphicsPresetUltra"),
            option(5, "hud.options.graphicsPresetAdvanced"),
        ]);
    }
    controls.push(choice(
        settings,
        "graphicsPreset",
        "hud.options.graphicsQuality",
        preset_options,
        true,
    ));
    if numeric_value(settings, "graphicsPreset").round() == 5.0 {
        for (key, label) in [
            ("terrainDetail", "hud.options.terrainDetail"),
            ("foliageDensity", "hud.options.foliageDensity"),
            ("effectsQuality", "hud.options.effectsQuality"),
            ("shadowQuality", "hud.options.shadowQuality"),
        ] {
            controls.push(choice(settings, key, label, low_high_options(), false));
        }
    }
    controls.push(choice(
        settings,
        "browserEffects",
        "hudChrome.options.browserEffects",
        vec![
            option(0, "hudChrome.options.browserEffectsAuto"),
            option(1, "hudChrome.options.browserEffectsFull"),
            option(2, "hudChrome.options.browserEffectsReduced"),
            option(3, "hudChrome.options.browserEffectsMinimal"),
        ],
        false,
    ));
    controls.push(note("hudChrome.options.browserEffectsNote"));
    if !environment.native_shell {
        controls.push(choice(
            settings,
            "interfaceMode",
            "hudChrome.options.interfaceMode",
            vec![
                option(0, "hudChrome.options.interfaceModeAuto"),
                option(1, "hudChrome.options.interfaceModeDesktop"),
                option(2, "hudChrome.options.interfaceModeTouch"),
            ],
            true,
        ));
        controls.push(note("hudChrome.options.interfaceModeNote"));
    }
    controls.push(slider(
        settings,
        "cameraSpeed",
        "hud.options.cameraSpeed",
        SliderFormat::Percent,
        0.05,
    ));
    if environment.touch {
        controls.push(slider(
            settings,
            "touchLookSpeed",
            "hud.options.touchLookSpeed",
            SliderFormat::Percent,
            0.05,
        ));
    }
    controls.extend([
        slider(
            settings,
            "brightness",
            "hud.options.brightness",
            SliderFormat::Percent,
            0.05,
        ),
        slider(
            settings,
            "cameraFov",
            "hud.options.fieldOfView",
            SliderFormat::Degrees,
            1.0,
        ),
        slider(
            settings,
            "renderScale",
            "hud.options.renderQuality",
            SliderFormat::Percent,
            0.05,
        ),
        numeric_toggle(settings, "fullscreen", "hud.options.fullscreen"),
        numeric_toggle(settings, "showOverflowXp", "game.settings.showOverflowXp"),
    ]);
    if environment.touch {
        controls.push(slider(
            settings,
            "touchOpacity",
            "hud.options.touchOpacity",
            SliderFormat::Percent,
            0.05,
        ));
    }
    controls.push(numeric_toggle(settings, "weather", "game.settings.weather"));
    if environment.touch {
        controls.extend([
            slider(
                settings,
                "joystickScale",
                "hud.options.joystickSize",
                SliderFormat::Percent,
                0.05,
            ),
            slider(
                settings,
                "actionButtonScale",
                "hud.options.buttonSize",
                SliderFormat::Percent,
                0.05,
            ),
            slider(
                settings,
                "joystickDeadzone",
                "hud.options.joystickDeadzone",
                SliderFormat::Percent,
                0.05,
            ),
            bool_toggle(settings, "touchInvertLook", "hud.options.invertLook"),
            bool_toggle(
                settings,
                "mobileCameraJoystick",
                "hudChrome.options.mobileCameraJoystick",
            ),
            bool_toggle(
                settings,
                "leftHandedTouch",
                "hudChrome.options.mobileLeftHanded",
            ),
        ]);
    }
    controls
}

pub fn build_audio_controls(settings: &ClientSettings) -> Vec<SettingsControl> {
    vec![
        slider(
            settings,
            "sfxVolume",
            "hud.options.soundEffects",
            SliderFormat::Percent,
            0.05,
        ),
        slider(
            settings,
            "musicVolume",
            "hud.options.musicVolume",
            SliderFormat::Percent,
            0.05,
        ),
        slider(
            settings,
            "voiceVolume",
            "hud.options.voiceVolume",
            SliderFormat::Percent,
            0.05,
        ),
        SettingsControl::MusicToggle {
            label_key: "hud.options.music",
        },
        bool_toggle(settings, "voiceEnabled", "hud.options.npcVoices"),
        bool_toggle(settings, "footstepSfx", "hudChrome.options.footstepSounds"),
        bool_toggle(
            settings,
            "interfaceSfx",
            "hudChrome.options.interfaceSounds",
        ),
        bool_toggle(settings, "clickFeedback", "hudChrome.options.clickFeedback"),
    ]
}

pub fn build_controller_controls(settings: &ClientSettings) -> Vec<SettingsControl> {
    vec![
        bool_toggle(settings, "gamepadEnabled", "hudChrome.controller.enable"),
        bool_toggle(settings, "gamepadInvertY", "hudChrome.controller.invertY"),
        slider(
            settings,
            "gamepadStickDeadzone",
            "hudChrome.controller.deadzone",
            SliderFormat::Percent,
            0.05,
        ),
        slider(
            settings,
            "gamepadCameraSpeed",
            "hudChrome.controller.cameraSpeed",
            SliderFormat::OneDecimal,
            0.05,
        ),
        slider(
            settings,
            "gamepadVibration",
            "hudChrome.controller.vibration",
            SliderFormat::Percent,
            0.05,
        ),
    ]
}

pub fn build_interface_controls(settings: &ClientSettings) -> Vec<SettingsControl> {
    let mut ui_scale = slider(
        settings,
        "uiScale",
        "hudChrome.options.uiScale",
        SliderFormat::Percent,
        0.05,
    );
    if let SettingsControl::Slider(control) = &mut ui_scale {
        control.commit_on_change = true;
    }
    vec![
        ui_scale,
        percent_slider(
            settings,
            "playerFrameScale",
            "hudChrome.options.playerFrameScale",
        ),
        percent_slider(
            settings,
            "targetFrameScale",
            "hudChrome.options.targetFrameScale",
        ),
        note("hudChrome.partyFrames.section"),
        choice(
            settings,
            "partyFrameStyle",
            "hudChrome.partyFrames.style",
            vec![
                option(0, "hudChrome.partyFrames.styleAutomatic"),
                option(1, "hudChrome.partyFrames.styleClassic"),
                option(2, "hudChrome.partyFrames.styleRaid"),
            ],
            false,
        ),
        percent_slider(settings, "partyFrameScale", "hudChrome.partyFrames.scale"),
        one_decimal_slider(
            settings,
            "partyFrameWidth",
            "hudChrome.partyFrames.width",
            5.0,
        ),
        one_decimal_slider(
            settings,
            "partyFrameHeight",
            "hudChrome.partyFrames.height",
            2.0,
        ),
        one_decimal_slider(
            settings,
            "partyFrameSpacing",
            "hudChrome.partyFrames.spacing",
            1.0,
        ),
        one_decimal_slider(
            settings,
            "partyFrameColumns",
            "hudChrome.partyFrames.columns",
            1.0,
        ),
        choice(
            settings,
            "partyFrameHealthText",
            "hudChrome.partyFrames.healthText",
            vec![
                option(0, "hudChrome.partyFrames.healthNone"),
                option(1, "hudChrome.partyFrames.healthPercent"),
                option(2, "hudChrome.partyFrames.healthCurrent"),
                option(3, "hudChrome.partyFrames.healthCurrentMax"),
            ],
            false,
        ),
        choice(
            settings,
            "partyFrameSort",
            "hudChrome.partyFrames.sort",
            vec![
                option(0, "hudChrome.partyFrames.sortGroup"),
                option(1, "hudChrome.partyFrames.sortRole"),
                option(2, "hudChrome.partyFrames.sortName"),
            ],
            false,
        ),
        bool_toggle(
            settings,
            "partyFrameShowResource",
            "hudChrome.partyFrames.showResource",
        ),
        bool_toggle(
            settings,
            "partyFrameShowAbsorbs",
            "hudChrome.partyFrames.showAbsorbs",
        ),
        bool_toggle(
            settings,
            "partyFrameShowAuras",
            "hudChrome.partyFrames.showAuras",
        ),
        bool_toggle(
            settings,
            "partyFrameShowSelf",
            "hudChrome.partyFrames.showSelf",
        ),
        percent_slider(settings, "hudOpacity", "hud.options.hudOpacity"),
        percent_slider(settings, "tooltipScale", "hud.options.tooltipScale"),
        percent_slider(settings, "fctScale", "hud.options.fctScale"),
        percent_slider(settings, "chatFontScale", "hud.options.chatFontScale"),
        percent_slider(settings, "chatOpacity", "hud.options.chatOpacity"),
        bool_toggle(settings, "compactChat", "hud.options.compactChat"),
        bool_toggle(settings, "frostedPanels", "hud.options.frostedPanels"),
        bool_toggle(settings, "highContrastText", "hud.options.highContrastText"),
        bool_toggle(settings, "reduceMotion", "hud.options.reduceMotion"),
        bool_toggle(
            settings,
            "showWalletOnCharacterScreen",
            "hudChrome.options.showWalletOnCharacterScreen",
        ),
        bool_toggle(
            settings,
            "showWalletOnPlayerCard",
            "hudChrome.options.showWalletOnPlayerCard",
        ),
        bool_toggle(settings, "showDevBadges", "hudChrome.options.showDevBadges"),
        bool_toggle(
            settings,
            "showOwnNameplate",
            "hudChrome.options.showOwnNameplate",
        ),
        bool_toggle(
            settings,
            "landingHighContrast",
            "hudChrome.options.highContrastBackground",
        ),
        bool_toggle(settings, "invertLookY", "hud.options.invertLookY"),
        bool_toggle(
            settings,
            "startAttackOnAbilityUse",
            "hudChrome.options.startAttackOnAbility",
        ),
        bool_toggle(
            settings,
            "showAttackButton",
            "hudChrome.options.showAttackButton",
        ),
        bool_toggle(
            settings,
            "walkByAutoloot",
            "hudChrome.options.walkByAutoloot",
        ),
        bool_toggle(settings, "groundReticle", "hudChrome.options.groundReticle"),
        bool_toggle(settings, "mouseoverCast", "hudChrome.options.mouseoverCast"),
        bool_toggle(
            settings,
            "aurasOnPlayerFrame",
            "hudChrome.options.aurasOnPlayerFrame",
        ),
        bool_toggle(settings, "showItemLevel", "hudChrome.options.showItemLevel"),
        bool_toggle(
            settings,
            "showSecondaryActionBar",
            "hudChrome.options.showSecondaryActionBar",
        ),
        bool_toggle(
            settings,
            "showTargetOfTarget",
            "hudChrome.options.showTargetOfTarget",
        ),
        bool_toggle(
            settings,
            "showAttackButton",
            "hudChrome.options.showAttackButton",
        ),
        bool_toggle(
            settings,
            "showDailyRewardsChest",
            "hudChrome.options.showDailyRewardsChest",
        ),
    ]
}

fn menu_entry(label_key: &'static str, panel: OptionsPanelId) -> OptionsMenuEntry {
    OptionsMenuEntry {
        label_key,
        action: OptionsMenuAction::GoTo(panel),
    }
}

fn slider(
    settings: &ClientSettings,
    key: &'static str,
    label_key: &'static str,
    format: SliderFormat,
    step: f64,
) -> SettingsControl {
    let spec = numeric_setting(key).expect("options slider must reference a numeric setting");
    SettingsControl::Slider(SliderControl {
        key,
        label_key,
        min: spec.min,
        max: spec.max,
        step,
        value: numeric_value(settings, key),
        format,
        commit_on_change: false,
    })
}

fn percent_slider(
    settings: &ClientSettings,
    key: &'static str,
    label_key: &'static str,
) -> SettingsControl {
    slider(settings, key, label_key, SliderFormat::Percent, 0.05)
}

fn one_decimal_slider(
    settings: &ClientSettings,
    key: &'static str,
    label_key: &'static str,
    step: f64,
) -> SettingsControl {
    slider(settings, key, label_key, SliderFormat::OneDecimal, step)
}

fn numeric_toggle(
    settings: &ClientSettings,
    key: &'static str,
    label_key: &'static str,
) -> SettingsControl {
    numeric_setting(key).expect("options toggle must reference a numeric setting");
    SettingsControl::NumericToggle(ToggleControl {
        key,
        label_key,
        on: numeric_toggle_is_on(numeric_value(settings, key)),
    })
}

fn bool_toggle(
    settings: &ClientSettings,
    key: &'static str,
    label_key: &'static str,
) -> SettingsControl {
    bool_setting(key).expect("options toggle must reference a boolean setting");
    SettingsControl::BoolToggle(ToggleControl {
        key,
        label_key,
        on: settings
            .boolean(key)
            .expect("registered boolean setting must have a current value"),
    })
}

fn choice(
    settings: &ClientSettings,
    key: &'static str,
    label_key: &'static str,
    options: Vec<ChoiceOption>,
    rerender: bool,
) -> SettingsControl {
    SettingsControl::Choice(ChoiceControl {
        key,
        label_key,
        current: numeric_value(settings, key).round() as i32,
        options,
        rerender,
    })
}

fn option(value: i32, label_key: &'static str) -> ChoiceOption {
    ChoiceOption { value, label_key }
}

fn low_high_options() -> Vec<ChoiceOption> {
    vec![
        option(0, "hud.options.terrainLow"),
        option(1, "hud.options.terrainHigh"),
    ]
}

fn note(text_key: &'static str) -> SettingsControl {
    SettingsControl::Note { text_key }
}

fn numeric_value(settings: &ClientSettings, key: &str) -> f64 {
    settings
        .numeric(key)
        .expect("registered numeric setting must have a current value")
}
