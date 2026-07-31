#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericSettingSpec {
    pub id: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoolSettingSpec {
    pub id: &'static str,
    pub default: bool,
}

const fn numeric(id: &'static str, min: f64, max: f64, default: f64) -> NumericSettingSpec {
    NumericSettingSpec {
        id,
        min,
        max,
        default,
    }
}

const fn boolean(id: &'static str, default: bool) -> BoolSettingSpec {
    BoolSettingSpec { id, default }
}

pub const NUMERIC_SETTINGS: [NumericSettingSpec; 43] = [
    numeric("cameraSpeed", 0.25, 1.25, 0.7),
    numeric("sfxVolume", 0.0, 1.0, 0.8),
    numeric("musicVolume", 0.0, 1.0, 0.8),
    numeric("voiceVolume", 0.0, 1.0, 0.9),
    numeric("brightness", 0.6, 1.5, 1.0),
    numeric("graphicsPreset", 1.0, 5.0, 2.0),
    numeric("browserEffects", 0.0, 3.0, 0.0),
    numeric("terrainDetail", 0.0, 1.0, 1.0),
    numeric("foliageDensity", 0.0, 1.0, 1.0),
    numeric("effectsQuality", 0.0, 1.0, 1.0),
    numeric("shadowQuality", 0.0, 1.0, 1.0),
    numeric("cameraFov", 55.0, 100.0, 60.0),
    numeric("renderScale", 0.5, 1.0, 1.0),
    numeric("fullscreen", 0.0, 1.0, 1.0),
    numeric("showOverflowXp", 0.0, 1.0, 1.0),
    numeric("clickToMove", 0.0, 1.0, 0.0),
    numeric("clickToMoveButton", 0.0, 2.0, 0.0),
    numeric("interfaceMode", 0.0, 2.0, 0.0),
    numeric("touchLookSpeed", 0.4, 1.8, 1.0),
    numeric("touchOpacity", 0.3, 1.0, 1.0),
    numeric("weather", 0.0, 1.0, 1.0),
    numeric("joystickScale", 0.7, 1.3, 1.0),
    numeric("actionButtonScale", 0.8, 1.3, 1.0),
    numeric("joystickDeadzone", 0.1, 0.4, 0.22),
    numeric("gamepadStickDeadzone", 0.05, 0.4, 0.18),
    numeric("gamepadCameraSpeed", 0.5, 5.0, 2.4),
    numeric("gamepadVibration", 0.0, 1.0, 1.0),
    numeric("tooltipScale", 0.85, 1.5, 1.0),
    numeric("chatFontScale", 0.85, 1.4, 1.0),
    numeric("chatOpacity", 0.3, 1.0, 1.0),
    numeric("fctScale", 0.7, 1.8, 1.0),
    numeric("hudOpacity", 0.5, 1.0, 1.0),
    numeric("uiScale", 0.85, 1.4, 1.0),
    numeric("playerFrameScale", 0.7, 1.15, 1.0),
    numeric("targetFrameScale", 0.7, 1.15, 1.0),
    numeric("partyFrameStyle", 0.0, 2.0, 0.0),
    numeric("partyFrameScale", 0.7, 1.4, 1.0),
    numeric("partyFrameWidth", 120.0, 260.0, 170.0),
    numeric("partyFrameHeight", 30.0, 72.0, 42.0),
    numeric("partyFrameSpacing", 0.0, 12.0, 4.0),
    numeric("partyFrameColumns", 1.0, 5.0, 1.0),
    numeric("partyFrameHealthText", 0.0, 3.0, 1.0),
    numeric("partyFrameSort", 0.0, 2.0, 0.0),
];

pub const BOOL_SETTINGS: [BoolSettingSpec; 41] = [
    boolean("mouseCamera", false),
    boolean("lockCursorOnRotate", true),
    boolean("gamepadEnabled", true),
    boolean("gamepadInvertY", false),
    boolean("leftHandedTouch", false),
    boolean("mobileCameraJoystick", false),
    boolean("filterProfanity", true),
    boolean("attackMove", false),
    boolean("touchInvertLook", false),
    boolean("startAttackOnAbilityUse", true),
    boolean("showAttackButton", true),
    boolean("walkByAutoloot", false),
    boolean("groundReticle", true),
    boolean("aurasOnPlayerFrame", false),
    boolean("mouseoverCast", true),
    boolean("partyFrameShowResource", true),
    boolean("partyFrameShowAbsorbs", true),
    boolean("partyFrameShowAuras", true),
    boolean("partyFrameShowSelf", false),
    boolean("reduceMotion", false),
    boolean("highContrastText", false),
    boolean("frostedPanels", false),
    boolean("compactChat", false),
    boolean("showFps", false),
    boolean("showWalletOnCharacterScreen", true),
    boolean("showWalletOnPlayerCard", true),
    boolean("showDevBadges", true),
    boolean("showOwnNameplate", true),
    boolean("invertLookY", false),
    boolean("voiceEnabled", true),
    boolean("footstepSfx", false),
    boolean("interfaceSfx", true),
    boolean("clickFeedback", true),
    boolean("landingHighContrast", false),
    boolean("questTrackerCollapsed", false),
    boolean("deedTrackerCollapsed", false),
    boolean("showItemLevel", false),
    boolean("showSecondaryActionBar", false),
    boolean("showTargetOfTarget", false),
    boolean("showDailyRewardsChest", true),
    boolean("graphicsDefaultApplied", false),
];

pub fn numeric_setting(id: &str) -> Option<&'static NumericSettingSpec> {
    NUMERIC_SETTINGS.iter().find(|setting| setting.id == id)
}

pub fn bool_setting(id: &str) -> Option<&'static BoolSettingSpec> {
    BOOL_SETTINGS.iter().find(|setting| setting.id == id)
}
