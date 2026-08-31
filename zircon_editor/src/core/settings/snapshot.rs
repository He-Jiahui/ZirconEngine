use std::sync::Arc;
use std::time::Duration;

use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use super::registry::SettingsRegistry;
use super::{
    EDITOR_AUTOSAVE_INTERVAL_SECS_KEY, EDITOR_COMMAND_PALETTE_MRU_KEY, EDITOR_DESIGN_TOKENS_KEY,
    EDITOR_KEYMAP_OVERRIDES_KEY, EDITOR_LOCALE_KEY, EditorCommandPaletteMru, EditorKeymapOverrides,
    SettingValue, SettingsCatalog, SettingsKey, VIEWPORT_ROTATE_STEP_DEGREES_KEY,
    VIEWPORT_SCALE_STEP_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
};

/// Parsed built-in keys retained by the registry at registration time.
///
/// Snapshot publication runs on every real settings mutation. Keeping the core keys here
/// prevents that hot path from allocating and reparsing static strings before looking up the
/// effective value.
#[derive(Clone, Default)]
pub(super) struct BuiltInSettingsSlots {
    design_tokens: Option<SettingsKey>,
    keymap_overrides: Option<SettingsKey>,
    command_palette_mru: Option<SettingsKey>,
    locale: Option<SettingsKey>,
    autosave_interval_secs: Option<SettingsKey>,
    viewport_translate_step: Option<SettingsKey>,
    viewport_rotate_step_degrees: Option<SettingsKey>,
    viewport_scale_step: Option<SettingsKey>,
}

impl BuiltInSettingsSlots {
    pub(super) fn record(&mut self, key: &SettingsKey) {
        let slot = match key.as_str() {
            EDITOR_DESIGN_TOKENS_KEY => &mut self.design_tokens,
            EDITOR_KEYMAP_OVERRIDES_KEY => &mut self.keymap_overrides,
            EDITOR_COMMAND_PALETTE_MRU_KEY => &mut self.command_palette_mru,
            EDITOR_LOCALE_KEY => &mut self.locale,
            EDITOR_AUTOSAVE_INTERVAL_SECS_KEY => &mut self.autosave_interval_secs,
            VIEWPORT_TRANSLATE_STEP_KEY => &mut self.viewport_translate_step,
            VIEWPORT_ROTATE_STEP_DEGREES_KEY => &mut self.viewport_rotate_step_degrees,
            VIEWPORT_SCALE_STEP_KEY => &mut self.viewport_scale_step,
            _ => return,
        };
        *slot = Some(key.clone());
    }

    fn design_tokens(&self) -> &SettingsKey {
        self.design_tokens
            .as_ref()
            .expect("the settings authority requires the design-token slot")
    }

    fn keymap_overrides(&self) -> &SettingsKey {
        self.keymap_overrides
            .as_ref()
            .expect("the settings authority requires the keymap-overrides slot")
    }

    pub(super) fn command_palette_mru(&self) -> &SettingsKey {
        self.command_palette_mru
            .as_ref()
            .expect("the settings authority requires the command-palette MRU slot")
    }

    fn locale(&self) -> &SettingsKey {
        self.locale
            .as_ref()
            .expect("the settings authority requires the locale slot")
    }

    fn autosave_interval_secs(&self) -> &SettingsKey {
        self.autosave_interval_secs
            .as_ref()
            .expect("the settings authority requires the autosave-interval slot")
    }

    fn viewport_translate_step(&self) -> &SettingsKey {
        self.viewport_translate_step
            .as_ref()
            .expect("the settings authority requires the viewport translate-step slot")
    }

    fn viewport_rotate_step_degrees(&self) -> &SettingsKey {
        self.viewport_rotate_step_degrees
            .as_ref()
            .expect("the settings authority requires the viewport rotate-step slot")
    }

    fn viewport_scale_step(&self) -> &SettingsKey {
        self.viewport_scale_step
            .as_ref()
            .expect("the settings authority requires the viewport scale-step slot")
    }
}

/// Typed viewport snap values resolved once for an immutable settings generation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportSnapSettings {
    translate_step: f64,
    rotate_step_degrees: f64,
    scale_step: f64,
}

impl ViewportSnapSettings {
    pub const fn translate_step(self) -> f64 {
        self.translate_step
    }

    pub const fn rotate_step_degrees(self) -> f64 {
        self.rotate_step_degrees
    }

    pub const fn scale_step(self) -> f64 {
        self.scale_step
    }
}

/// Immutable, typed values published by the sole settings authority for one generation.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingsSnapshot {
    generation: u64,
    catalog: Arc<SettingsCatalog>,
    design_tokens: Arc<EditorDesignTokens>,
    keymap_overrides: Arc<EditorKeymapOverrides>,
    command_palette_mru: Arc<EditorCommandPaletteMru>,
    locale: Arc<str>,
    autosave_interval: Duration,
    viewport_snap: ViewportSnapSettings,
}

impl SettingsSnapshot {
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn catalog(&self) -> &SettingsCatalog {
        self.catalog.as_ref()
    }

    pub fn shares_catalog_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.catalog, &other.catalog)
    }

    pub(crate) fn catalog_handle(&self) -> Arc<SettingsCatalog> {
        Arc::clone(&self.catalog)
    }

    pub(crate) fn shares_catalog_handle_with(&self, catalog: &Arc<SettingsCatalog>) -> bool {
        Arc::ptr_eq(&self.catalog, catalog)
    }

    pub fn design_tokens(&self) -> &EditorDesignTokens {
        self.design_tokens.as_ref()
    }

    /// Shares the immutable token payload with UI projections that only refresh on token changes.
    pub(crate) fn design_tokens_handle(&self) -> Arc<EditorDesignTokens> {
        Arc::clone(&self.design_tokens)
    }

    pub fn keymap_overrides(&self) -> &EditorKeymapOverrides {
        self.keymap_overrides.as_ref()
    }

    /// Shares the immutable override payload with projections that must avoid rebuilding on
    /// unrelated settings generations.
    pub(crate) fn keymap_overrides_handle(&self) -> Arc<EditorKeymapOverrides> {
        Arc::clone(&self.keymap_overrides)
    }

    pub fn command_palette_mru(&self) -> &EditorCommandPaletteMru {
        self.command_palette_mru.as_ref()
    }

    /// The resolved User language setting for consumers that render localizable editor content.
    pub fn locale(&self) -> &str {
        self.locale.as_ref()
    }

    /// The effective user-configured autosave cadence for the active editor process.
    pub const fn autosave_interval(&self) -> Duration {
        self.autosave_interval
    }

    pub const fn viewport_snap(&self) -> ViewportSnapSettings {
        self.viewport_snap
    }

    pub(super) fn from_registry(registry: &SettingsRegistry) -> Self {
        let slots = &registry.built_in_slots;
        Self {
            generation: registry.revision,
            catalog: Arc::new(SettingsCatalog::from_registry(registry)),
            design_tokens: Arc::new(built_in_design_tokens(registry, slots.design_tokens())),
            keymap_overrides: Arc::new(built_in_keymap_overrides(
                registry,
                slots.keymap_overrides(),
            )),
            command_palette_mru: Arc::new(built_in_command_palette_mru(
                registry,
                slots.command_palette_mru(),
            )),
            locale: Arc::from(built_in_locale(registry, slots.locale())),
            autosave_interval: built_in_autosave_interval(registry, slots.autosave_interval_secs()),
            viewport_snap: ViewportSnapSettings {
                translate_step: built_in_float(registry, slots.viewport_translate_step()),
                rotate_step_degrees: built_in_float(registry, slots.viewport_rotate_step_degrees()),
                scale_step: built_in_float(registry, slots.viewport_scale_step()),
            },
        }
    }

    pub(super) fn after_change(
        previous: &SettingsSnapshot,
        registry: &SettingsRegistry,
        change: &super::SettingChange,
    ) -> Self {
        let slots = &registry.built_in_slots;
        let mut snapshot = Self {
            generation: registry.revision,
            catalog: Arc::clone(&previous.catalog),
            design_tokens: Arc::clone(&previous.design_tokens),
            keymap_overrides: Arc::clone(&previous.keymap_overrides),
            command_palette_mru: Arc::clone(&previous.command_palette_mru),
            locale: Arc::clone(&previous.locale),
            autosave_interval: previous.autosave_interval,
            viewport_snap: previous.viewport_snap,
        };
        if &change.key == slots.design_tokens() {
            snapshot.design_tokens =
                Arc::new(built_in_design_tokens(registry, slots.design_tokens()));
        } else if &change.key == slots.keymap_overrides() {
            snapshot.keymap_overrides = Arc::new(built_in_keymap_overrides(
                registry,
                slots.keymap_overrides(),
            ));
        } else if &change.key == slots.command_palette_mru() {
            snapshot.command_palette_mru = Arc::new(built_in_command_palette_mru(
                registry,
                slots.command_palette_mru(),
            ));
        } else if &change.key == slots.locale() {
            snapshot.locale = Arc::from(built_in_locale(registry, slots.locale()));
        } else if &change.key == slots.autosave_interval_secs() {
            snapshot.autosave_interval =
                built_in_autosave_interval(registry, slots.autosave_interval_secs());
        } else if &change.key == slots.viewport_translate_step() {
            snapshot.viewport_snap.translate_step =
                built_in_float(registry, slots.viewport_translate_step());
        } else if &change.key == slots.viewport_rotate_step_degrees() {
            snapshot.viewport_snap.rotate_step_degrees =
                built_in_float(registry, slots.viewport_rotate_step_degrees());
        } else if &change.key == slots.viewport_scale_step() {
            snapshot.viewport_snap.scale_step =
                built_in_float(registry, slots.viewport_scale_step());
        }
        snapshot
    }
}

fn built_in_design_tokens(registry: &SettingsRegistry, key: &SettingsKey) -> EditorDesignTokens {
    match registry
        .resolve(key)
        .expect("the built-in design-token setting is registered")
    {
        SettingValue::DesignTokens(tokens) => tokens.clone(),
        _ => unreachable!("the built-in design-token setting has a design-token schema"),
    }
}

fn built_in_keymap_overrides(
    registry: &SettingsRegistry,
    key: &SettingsKey,
) -> EditorKeymapOverrides {
    match registry
        .resolve(key)
        .expect("the built-in keymap-overrides setting is registered")
    {
        SettingValue::KeymapOverrides(overrides) => overrides.clone(),
        _ => unreachable!("the built-in keymap-overrides setting has a keymap-overrides schema"),
    }
}

fn built_in_command_palette_mru(
    registry: &SettingsRegistry,
    key: &SettingsKey,
) -> EditorCommandPaletteMru {
    match registry
        .resolve(key)
        .expect("the built-in command-palette MRU setting is registered")
    {
        SettingValue::CommandPaletteMru(mru) => mru.clone(),
        _ => unreachable!("the built-in command-palette MRU key has an MRU schema"),
    }
}

fn built_in_float(registry: &SettingsRegistry, key: &SettingsKey) -> f64 {
    match registry
        .resolve(key)
        .expect("the built-in viewport snap key is registered")
    {
        SettingValue::Float(value) => *value,
        _ => unreachable!("the built-in viewport snap key uses a float schema"),
    }
}

fn built_in_locale(registry: &SettingsRegistry, key: &SettingsKey) -> String {
    match registry
        .resolve(key)
        .expect("the built-in locale setting is registered")
    {
        SettingValue::Enum(value) => value.clone(),
        _ => unreachable!("the built-in locale setting uses an enum schema"),
    }
}

fn built_in_autosave_interval(registry: &SettingsRegistry, key: &SettingsKey) -> Duration {
    match registry
        .resolve(key)
        .expect("the built-in autosave interval setting is registered")
    {
        SettingValue::Int(seconds) => Duration::from_secs(
            u64::try_from(*seconds)
                .expect("the autosave-interval setting schema only permits positive seconds"),
        ),
        _ => unreachable!("the built-in autosave interval setting uses an integer schema"),
    }
}
