#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSettingApplication {
    MouseCamera,
    LockCursorOnRotate,
    CameraSpeed,
    TouchLookSpeed,
    TouchInvertLook,
    InvertLookY,
    ClickMove,
    ClickMoveButton,
    AttackMove,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSettingApplication {
    AmbientVolume,
    SoundEffectsVolume,
    MusicVolume,
    VoiceVolume,
    VoiceEnabled,
    FootstepsEnabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererSettingApplication {
    Brightness,
    CameraFov,
    RenderScale,
    WeatherEnabled,
    ShowDevBadges,
    ShowOwnNameplate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadSettingApplication {
    Enabled,
    InvertY,
    StickDeadzone,
    CameraSpeed,
    Vibration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchSettingApplication {
    CameraJoystickEnabled,
    InterfaceMode,
    MoveDeadzone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudSettingApplication {
    CancelGroundAimWhenDisabled,
    AurasOnPlayerFrame,
    DailyRewardsChestVisible,
    ReapplySavedGeometry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiEffectsApplication {
    ApplyNow,
    ApplyDebounced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingApplication {
    Input(InputSettingApplication),
    Audio(AudioSettingApplication),
    Renderer(RendererSettingApplication),
    Gamepad(GamepadSettingApplication),
    Touch(TouchSettingApplication),
    Hud(HudSettingApplication),
    UiEffects(UiEffectsApplication),
    RootCssVariable(&'static str),
    ElementCssVariable {
        element_id: &'static str,
        property: &'static str,
    },
    BodyClass(&'static str),
    BrowserEffects,
    Fullscreen,
    PerformanceOverlay,
    WalletCharacterScreen,
    LandingBackdrop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingApplicationRoute {
    pub key: &'static str,
    pub applications: &'static [SettingApplication],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClientSettingValue {
    Numeric(f64),
    Boolean(bool),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientSettingChange {
    pub key: &'static str,
    pub value: ClientSettingValue,
    pub applications: &'static [SettingApplication],
}

const fn route(
    key: &'static str,
    applications: &'static [SettingApplication],
) -> SettingApplicationRoute {
    SettingApplicationRoute { key, applications }
}

pub const NUMERIC_SETTING_APPLICATIONS: [SettingApplicationRoute; 43] = [
    route(
        "cameraSpeed",
        &[SettingApplication::Input(
            InputSettingApplication::CameraSpeed,
        )],
    ),
    route(
        "sfxVolume",
        &[
            SettingApplication::Audio(AudioSettingApplication::AmbientVolume),
            SettingApplication::Audio(AudioSettingApplication::SoundEffectsVolume),
        ],
    ),
    route(
        "musicVolume",
        &[SettingApplication::Audio(
            AudioSettingApplication::MusicVolume,
        )],
    ),
    route(
        "voiceVolume",
        &[SettingApplication::Audio(
            AudioSettingApplication::VoiceVolume,
        )],
    ),
    route(
        "brightness",
        &[SettingApplication::Renderer(
            RendererSettingApplication::Brightness,
        )],
    ),
    route(
        "graphicsPreset",
        &[SettingApplication::UiEffects(
            UiEffectsApplication::ApplyNow,
        )],
    ),
    route("browserEffects", &[SettingApplication::BrowserEffects]),
    route("terrainDetail", &[]),
    route("foliageDensity", &[]),
    route(
        "effectsQuality",
        &[SettingApplication::UiEffects(
            UiEffectsApplication::ApplyDebounced,
        )],
    ),
    route("shadowQuality", &[]),
    route(
        "cameraFov",
        &[SettingApplication::Renderer(
            RendererSettingApplication::CameraFov,
        )],
    ),
    route(
        "renderScale",
        &[SettingApplication::Renderer(
            RendererSettingApplication::RenderScale,
        )],
    ),
    route("fullscreen", &[SettingApplication::Fullscreen]),
    route("showOverflowXp", &[]),
    route(
        "clickToMove",
        &[SettingApplication::Input(
            InputSettingApplication::ClickMove,
        )],
    ),
    route(
        "clickToMoveButton",
        &[SettingApplication::Input(
            InputSettingApplication::ClickMoveButton,
        )],
    ),
    route(
        "interfaceMode",
        &[SettingApplication::Touch(
            TouchSettingApplication::InterfaceMode,
        )],
    ),
    route(
        "touchLookSpeed",
        &[SettingApplication::Input(
            InputSettingApplication::TouchLookSpeed,
        )],
    ),
    route(
        "touchOpacity",
        &[SettingApplication::RootCssVariable("--touch-opacity")],
    ),
    route(
        "weather",
        &[SettingApplication::Renderer(
            RendererSettingApplication::WeatherEnabled,
        )],
    ),
    route(
        "joystickScale",
        &[SettingApplication::ElementCssVariable {
            element_id: "mobile-controls",
            property: "--joy-scale",
        }],
    ),
    route(
        "actionButtonScale",
        &[SettingApplication::ElementCssVariable {
            element_id: "mobile-controls",
            property: "--btn-scale",
        }],
    ),
    route(
        "joystickDeadzone",
        &[SettingApplication::Touch(
            TouchSettingApplication::MoveDeadzone,
        )],
    ),
    route(
        "gamepadStickDeadzone",
        &[SettingApplication::Gamepad(
            GamepadSettingApplication::StickDeadzone,
        )],
    ),
    route(
        "gamepadCameraSpeed",
        &[SettingApplication::Gamepad(
            GamepadSettingApplication::CameraSpeed,
        )],
    ),
    route(
        "gamepadVibration",
        &[SettingApplication::Gamepad(
            GamepadSettingApplication::Vibration,
        )],
    ),
    route(
        "tooltipScale",
        &[SettingApplication::RootCssVariable("--tooltip-scale")],
    ),
    route(
        "chatFontScale",
        &[SettingApplication::RootCssVariable("--chat-font-scale")],
    ),
    route(
        "chatOpacity",
        &[SettingApplication::RootCssVariable("--chat-opacity")],
    ),
    route(
        "fctScale",
        &[SettingApplication::RootCssVariable("--fct-scale")],
    ),
    route(
        "hudOpacity",
        &[SettingApplication::RootCssVariable("--hud-opacity")],
    ),
    route(
        "uiScale",
        &[
            SettingApplication::RootCssVariable("--ui-scale"),
            SettingApplication::Hud(HudSettingApplication::ReapplySavedGeometry),
        ],
    ),
    route(
        "playerFrameScale",
        &[SettingApplication::RootCssVariable("--player-frame-scale")],
    ),
    route(
        "targetFrameScale",
        &[SettingApplication::RootCssVariable("--target-frame-scale")],
    ),
    route("partyFrameStyle", &[]),
    route("partyFrameScale", &[]),
    route("partyFrameWidth", &[]),
    route("partyFrameHeight", &[]),
    route("partyFrameSpacing", &[]),
    route("partyFrameColumns", &[]),
    route("partyFrameHealthText", &[]),
    route("partyFrameSort", &[]),
];

pub const BOOL_SETTING_APPLICATIONS: [SettingApplicationRoute; 41] = [
    route(
        "mouseCamera",
        &[SettingApplication::Input(
            InputSettingApplication::MouseCamera,
        )],
    ),
    route(
        "lockCursorOnRotate",
        &[SettingApplication::Input(
            InputSettingApplication::LockCursorOnRotate,
        )],
    ),
    route(
        "gamepadEnabled",
        &[SettingApplication::Gamepad(
            GamepadSettingApplication::Enabled,
        )],
    ),
    route(
        "gamepadInvertY",
        &[SettingApplication::Gamepad(
            GamepadSettingApplication::InvertY,
        )],
    ),
    route(
        "leftHandedTouch",
        &[SettingApplication::BodyClass("mobile-left-handed")],
    ),
    route(
        "mobileCameraJoystick",
        &[
            SettingApplication::BodyClass("mobile-camera-joystick-on"),
            SettingApplication::Touch(TouchSettingApplication::CameraJoystickEnabled),
        ],
    ),
    route("filterProfanity", &[]),
    route(
        "attackMove",
        &[SettingApplication::Input(
            InputSettingApplication::AttackMove,
        )],
    ),
    route(
        "touchInvertLook",
        &[SettingApplication::Input(
            InputSettingApplication::TouchInvertLook,
        )],
    ),
    route("startAttackOnAbilityUse", &[]),
    route("showAttackButton", &[]),
    route("walkByAutoloot", &[]),
    route(
        "groundReticle",
        &[SettingApplication::Hud(
            HudSettingApplication::CancelGroundAimWhenDisabled,
        )],
    ),
    route(
        "aurasOnPlayerFrame",
        &[SettingApplication::Hud(
            HudSettingApplication::AurasOnPlayerFrame,
        )],
    ),
    route("mouseoverCast", &[]),
    route("partyFrameShowResource", &[]),
    route("partyFrameShowAbsorbs", &[]),
    route("partyFrameShowAuras", &[]),
    route("partyFrameShowSelf", &[]),
    route(
        "reduceMotion",
        &[
            SettingApplication::BodyClass("reduce-motion"),
            SettingApplication::UiEffects(UiEffectsApplication::ApplyNow),
        ],
    ),
    route(
        "highContrastText",
        &[SettingApplication::BodyClass("high-contrast-text")],
    ),
    route(
        "frostedPanels",
        &[SettingApplication::BodyClass("frosted-panels")],
    ),
    route(
        "compactChat",
        &[SettingApplication::BodyClass("compact-chat")],
    ),
    route("showFps", &[SettingApplication::PerformanceOverlay]),
    route(
        "showWalletOnCharacterScreen",
        &[SettingApplication::WalletCharacterScreen],
    ),
    route("showWalletOnPlayerCard", &[]),
    route(
        "showDevBadges",
        &[SettingApplication::Renderer(
            RendererSettingApplication::ShowDevBadges,
        )],
    ),
    route(
        "showOwnNameplate",
        &[SettingApplication::Renderer(
            RendererSettingApplication::ShowOwnNameplate,
        )],
    ),
    route(
        "invertLookY",
        &[SettingApplication::Input(
            InputSettingApplication::InvertLookY,
        )],
    ),
    route(
        "voiceEnabled",
        &[SettingApplication::Audio(
            AudioSettingApplication::VoiceEnabled,
        )],
    ),
    route(
        "footstepSfx",
        &[SettingApplication::Audio(
            AudioSettingApplication::FootstepsEnabled,
        )],
    ),
    route("interfaceSfx", &[]),
    route("clickFeedback", &[]),
    route(
        "landingHighContrast",
        &[SettingApplication::LandingBackdrop],
    ),
    route("questTrackerCollapsed", &[]),
    route("deedTrackerCollapsed", &[]),
    route("showItemLevel", &[]),
    route(
        "showSecondaryActionBar",
        &[SettingApplication::BodyClass("show-actionbar2")],
    ),
    route("showTargetOfTarget", &[]),
    route(
        "showDailyRewardsChest",
        &[SettingApplication::Hud(
            HudSettingApplication::DailyRewardsChestVisible,
        )],
    ),
    route("graphicsDefaultApplied", &[]),
];

pub fn setting_application_route(key: &str) -> Option<&'static SettingApplicationRoute> {
    NUMERIC_SETTING_APPLICATIONS
        .iter()
        .chain(BOOL_SETTING_APPLICATIONS.iter())
        .find(|route| route.key == key)
}

pub fn normalized_numeric_setting_change(key: &str, value: f64) -> Option<ClientSettingChange> {
    let setting = numeric_setting(key)?;
    Some(change(setting.id, ClientSettingValue::Numeric(value)))
}

pub fn normalized_boolean_setting_change(key: &str, value: bool) -> Option<ClientSettingChange> {
    let setting = bool_setting(key)?;
    Some(change(setting.id, ClientSettingValue::Boolean(value)))
}

pub fn client_settings_application_plan(settings: &ClientSettings) -> Vec<ClientSettingChange> {
    NUMERIC_SETTINGS
        .iter()
        .map(|setting| {
            change(
                setting.id,
                ClientSettingValue::Numeric(
                    settings.numeric(setting.id).unwrap_or(setting.default),
                ),
            )
        })
        .chain(BOOL_SETTINGS.iter().map(|setting| {
            change(
                setting.id,
                ClientSettingValue::Boolean(
                    settings.boolean(setting.id).unwrap_or(setting.default),
                ),
            )
        }))
        .collect()
}

fn change(key: &'static str, value: ClientSettingValue) -> ClientSettingChange {
    ClientSettingChange {
        key,
        value,
        applications: setting_application_route(key)
            .expect("registered client setting must have an application route")
            .applications,
    }
}
use super::{
    registry::{bool_setting, numeric_setting, BOOL_SETTINGS, NUMERIC_SETTINGS},
    state::ClientSettings,
};
