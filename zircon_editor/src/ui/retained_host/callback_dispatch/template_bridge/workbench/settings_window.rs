use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime_interface::ui::component::UiValue;

use crate::core::settings::{
    ResolvedSettingValue, ResolvedSettingsBatch, SettingSchema, SettingValue, SettingValueSource,
    SettingsScope,
};
use crate::ui::settings::{
    SettingsLocalizationDomain, SettingsPersistenceHealthProjection, SettingsWindowProjection,
};
use crate::ui::template_runtime::RetainedUiHostValue;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

pub(crate) const WORKBENCH_SETTINGS_WINDOW_CONTROL_ID: &str = "WorkbenchPreferences";

const OPEN: &str = "open";
const POPUP_OPEN: &str = "popup_open";
const FOCUSED: &str = "focused";
const SELECTED: &str = "selected";
const TITLE: &str = "title";
const LOCALE: &str = "locale";
const SETTINGS_GENERATION: &str = "settings_generation";
const CONTRIBUTION_GENERATION: &str = "contribution_generation";
const ENABLED_CAPABILITIES: &str = "enabled_capabilities";
const CATEGORIES: &str = "categories";
const SETTINGS: &str = "settings";
const SETTINGS_VALUES: &str = "settings_values";
const PLUGIN_PAGES: &str = "plugin_pages";
const SELECTED_CATEGORY_ID: &str = "selected_category_id";
const SETTINGS_EDITOR_OPEN_KEY: &str = "settings_editor_open_key";
const SETTINGS_EDITOR_OPEN_KIND: &str = "settings_editor_open_kind";
const SETTINGS_CATEGORY_SCROLL_OFFSET: &str = "settings_category_scroll_offset";
const SETTINGS_SCROLL_OFFSET: &str = "settings_scroll_offset";
const SETTINGS_PERSISTENCE_HEALTH_GENERATION: &str = "settings_persistence_health_generation";
const SETTINGS_PERSISTENCE_RETRY_SCOPE: &str = "settings_persistence_retry_scope";
const SETTINGS_PERSISTENCE_STATUS_TEXT: &str = "settings_persistence_status_text";

pub(crate) struct WorkbenchSettingsOpenState {
    title: String,
    locale: String,
    settings_generation: u64,
    contribution_generation: u64,
    enabled_capabilities: UiValue,
    categories: UiValue,
    settings: UiValue,
    settings_values: UiValue,
    plugin_pages: UiValue,
    selected_category_id: String,
    settings_persistence_health_generation: u64,
    settings_persistence_retry_scope: String,
    settings_persistence_status_text: String,
}

pub(crate) struct WorkbenchSettingsWindowRevision {
    pub(crate) settings_generation: u64,
    pub(crate) contribution_generation: u64,
    pub(crate) enabled_capabilities: Vec<String>,
    pub(crate) locale: String,
    pub(crate) selected_category_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchSettingsEditorKind {
    Enum,
    Color,
}

impl WorkbenchSettingsEditorKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Enum => "enum",
            Self::Color => "color",
        }
    }
}

impl WorkbenchSettingsOpenState {
    pub(crate) fn initial_category_id(projection: &SettingsWindowProjection) -> String {
        default_category_identity(projection)
    }

    pub(crate) fn retain_category_id(
        projection: &SettingsWindowProjection,
        current_category_id: &str,
    ) -> String {
        projection
            .categories()
            .iter()
            .map(category_identity)
            .find(|category_id| category_id == current_category_id)
            .unwrap_or_else(|| default_category_identity(projection))
    }

    pub(crate) fn from_projection(
        projection: &SettingsWindowProjection,
        selected_category_id: String,
        values: &ResolvedSettingsBatch,
        health: &SettingsPersistenceHealthProjection,
    ) -> Self {
        Self {
            title: projection.title().to_owned(),
            locale: projection.locale().as_str().to_owned(),
            settings_generation: values.generation(),
            contribution_generation: projection.contribution_generation(),
            enabled_capabilities: UiValue::Array(
                projection
                    .enabled_capabilities()
                    .map(|capability| UiValue::String(capability.to_owned()))
                    .collect(),
            ),
            categories: UiValue::Array(
                projection.categories().iter().map(category_value).collect(),
            ),
            settings: UiValue::Array(projection.settings().iter().map(setting_value).collect()),
            settings_values: settings_value_payload(values),
            plugin_pages: UiValue::Array(
                projection
                    .plugin_pages()
                    .iter()
                    .map(plugin_page_value)
                    .collect(),
            ),
            selected_category_id,
            settings_persistence_health_generation: health.generation(),
            settings_persistence_retry_scope: health.retry_scope_name().to_owned(),
            settings_persistence_status_text: health.status_text().to_owned(),
        }
    }
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn open_settings_window(
        &mut self,
        state: WorkbenchSettingsOpenState,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.has_control(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID) {
            return Ok(false);
        }
        self.set_visible(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, true)?;
        for property in [OPEN, POPUP_OPEN, FOCUSED, SELECTED] {
            self.mutate_control_property(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                property,
                UiValue::Bool(true),
            )?;
        }
        self.write_settings_window_state(state)?;
        for property in [SETTINGS_CATEGORY_SCROLL_OFFSET, SETTINGS_SCROLL_OFFSET] {
            self.mutate_control_property(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                property,
                UiValue::Float(0.0),
            )?;
        }
        self.clear_settings_editor_open()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn refresh_settings_window(
        &mut self,
        state: WorkbenchSettingsOpenState,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN) {
            return Ok(false);
        }
        self.write_settings_window_state(state)?;
        self.clear_settings_editor_open()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn refresh_settings_values(
        &mut self,
        values: &ResolvedSettingsBatch,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN) {
            return Ok(false);
        }
        self.write_settings_values(values)?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn refresh_settings_values_and_close_editor(
        &mut self,
        values: &ResolvedSettingsBatch,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN) {
            return Ok(false);
        }
        self.write_settings_values(values)?;
        self.clear_settings_editor_open()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn settings_window_revision(&self) -> Option<WorkbenchSettingsWindowRevision> {
        self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN)
            .then(|| WorkbenchSettingsWindowRevision {
                settings_generation: control_generation(
                    self.control_integer(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, SETTINGS_GENERATION),
                ),
                contribution_generation: control_generation(self.control_integer(
                    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                    CONTRIBUTION_GENERATION,
                )),
                enabled_capabilities: self.control_string_array(
                    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                    ENABLED_CAPABILITIES,
                ),
                locale: self
                    .control_string(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, LOCALE)
                    .unwrap_or_default(),
                selected_category_id: self
                    .control_string(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, SELECTED_CATEGORY_ID)
                    .unwrap_or_default(),
            })
    }

    pub(crate) fn settings_editor_open_key(
        &self,
        kind: WorkbenchSettingsEditorKind,
    ) -> Option<String> {
        (self
            .control_string(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                SETTINGS_EDITOR_OPEN_KIND,
            )
            .as_deref()
            == Some(kind.as_str()))
        .then(|| {
            self.control_string(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                SETTINGS_EDITOR_OPEN_KEY,
            )
        })
        .flatten()
        .filter(|key| !key.is_empty())
    }

    pub(crate) fn prepare_settings_persistence_health(
        &mut self,
        health: &SettingsPersistenceHealthProjection,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN) {
            return Ok(false);
        }
        let retry_scope = health.retry_scope_name();
        if control_generation(self.control_integer(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            SETTINGS_PERSISTENCE_HEALTH_GENERATION,
        )) == health.generation()
            && self
                .control_string(
                    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                    SETTINGS_PERSISTENCE_RETRY_SCOPE,
                )
                .as_deref()
                == Some(retry_scope)
            && self
                .control_string(
                    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                    SETTINGS_PERSISTENCE_STATUS_TEXT,
                )
                .as_deref()
                == Some(health.status_text())
        {
            return Ok(false);
        }
        for (property, value) in [
            (
                SETTINGS_PERSISTENCE_HEALTH_GENERATION,
                generation_value(health.generation()),
            ),
            (
                SETTINGS_PERSISTENCE_RETRY_SCOPE,
                UiValue::String(retry_scope.to_owned()),
            ),
            (
                SETTINGS_PERSISTENCE_STATUS_TEXT,
                UiValue::String(health.status_text().to_owned()),
            ),
        ] {
            self.mutate_control_property(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, property, value)?;
        }
        Ok(true)
    }

    pub(crate) fn toggle_settings_editor(
        &mut self,
        setting_key: &str,
        kind: WorkbenchSettingsEditorKind,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if setting_key.is_empty() || !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN)
        {
            return Ok(false);
        }
        let open = self.settings_editor_open_key(kind).as_deref() != Some(setting_key);
        if open {
            self.set_settings_editor_open(setting_key, kind.as_str())?;
        } else {
            self.clear_settings_editor_open()?;
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn close_settings_editor(
        &mut self,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if self
            .control_string(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                SETTINGS_EDITOR_OPEN_KEY,
            )
            .is_none_or(|key| key.is_empty())
        {
            return Ok(false);
        }
        self.clear_settings_editor_open()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    fn set_settings_editor_open(
        &mut self,
        setting_key: &str,
        kind: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            SETTINGS_EDITOR_OPEN_KEY,
            UiValue::String(setting_key.to_owned()),
        )?;
        self.mutate_control_property(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            SETTINGS_EDITOR_OPEN_KIND,
            UiValue::String(kind.to_owned()),
        )
    }

    fn clear_settings_editor_open(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.set_settings_editor_open("", "")
    }

    fn write_settings_values(
        &mut self,
        values: &ResolvedSettingsBatch,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            SETTINGS_GENERATION,
            generation_value(values.generation()),
        )?;
        self.mutate_control_property(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            SETTINGS_VALUES,
            settings_value_payload(values),
        )
    }

    fn write_settings_window_state(
        &mut self,
        state: WorkbenchSettingsOpenState,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for (property, value) in [
            (TITLE, UiValue::String(state.title)),
            (LOCALE, UiValue::String(state.locale)),
            (
                SETTINGS_GENERATION,
                generation_value(state.settings_generation),
            ),
            (
                CONTRIBUTION_GENERATION,
                generation_value(state.contribution_generation),
            ),
            (ENABLED_CAPABILITIES, state.enabled_capabilities),
            (CATEGORIES, state.categories),
            (SETTINGS, state.settings),
            (SETTINGS_VALUES, state.settings_values),
            (PLUGIN_PAGES, state.plugin_pages),
            (
                SELECTED_CATEGORY_ID,
                UiValue::String(state.selected_category_id),
            ),
            (
                SETTINGS_PERSISTENCE_HEALTH_GENERATION,
                generation_value(state.settings_persistence_health_generation),
            ),
            (
                SETTINGS_PERSISTENCE_RETRY_SCOPE,
                UiValue::String(state.settings_persistence_retry_scope),
            ),
            (
                SETTINGS_PERSISTENCE_STATUS_TEXT,
                UiValue::String(state.settings_persistence_status_text),
            ),
        ] {
            self.mutate_control_property(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, property, value)?;
        }
        Ok(())
    }

    pub(crate) fn close_settings_window(
        &mut self,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN)
            && !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, POPUP_OPEN)
        {
            return Ok(false);
        }
        self.set_visible(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, false)?;
        for property in [OPEN, POPUP_OPEN, FOCUSED, SELECTED] {
            self.mutate_control_property(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                property,
                UiValue::Bool(false),
            )?;
        }
        self.clear_settings_editor_open()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn update_settings_scroll_offsets(
        &mut self,
        category_scroll_offset: f32,
        setting_scroll_offset: f32,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN)
            || !category_scroll_offset.is_finite()
            || !setting_scroll_offset.is_finite()
        {
            return Ok(false);
        }
        let category_scroll_offset = category_scroll_offset.max(0.0);
        let setting_scroll_offset = setting_scroll_offset.max(0.0);
        let current_category = self
            .control_float(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                SETTINGS_CATEGORY_SCROLL_OFFSET,
            )
            .unwrap_or_default();
        let current_setting = self
            .control_float(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, SETTINGS_SCROLL_OFFSET)
            .unwrap_or_default();
        if (current_category - category_scroll_offset).abs() <= f32::EPSILON
            && (current_setting - setting_scroll_offset).abs() <= f32::EPSILON
        {
            return Ok(false);
        }
        for (property, value) in [
            (SETTINGS_CATEGORY_SCROLL_OFFSET, category_scroll_offset),
            (SETTINGS_SCROLL_OFFSET, setting_scroll_offset),
        ] {
            self.mutate_control_property(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                property,
                UiValue::Float(value as f64),
            )?;
        }
        self.clear_settings_editor_open()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(crate) fn select_settings_category(
        &mut self,
        control_id: &str,
        category_id: &str,
        values: &ResolvedSettingsBatch,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        if control_id != WORKBENCH_SETTINGS_WINDOW_CONTROL_ID {
            return Ok(None);
        }
        if category_id.is_empty() || !self.settings_category_exists(category_id) {
            return Ok(Some(false));
        }
        self.mutate_control_property(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            SELECTED_CATEGORY_ID,
            UiValue::String(category_id.to_owned()),
        )?;
        self.mutate_control_property(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            SETTINGS_SCROLL_OFFSET,
            UiValue::Float(0.0),
        )?;
        self.clear_settings_editor_open()?;
        self.mutate_control_property(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            SETTINGS_GENERATION,
            generation_value(values.generation()),
        )?;
        self.mutate_control_property(
            WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
            SETTINGS_VALUES,
            settings_value_payload(values),
        )?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }

    fn settings_category_exists(&self, category_id: &str) -> bool {
        let Some(RetainedUiHostValue::Array(categories)) = self
            .host_projection()
            .node_by_control_id(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID)
            .and_then(|node| node.properties.get(CATEGORIES))
        else {
            return false;
        };
        categories.iter().any(|category| {
            let RetainedUiHostValue::Table(category) = category else {
                return false;
            };
            let domain = host_string(category.get("domain"));
            let key_path = host_string(category.get("key_path"));
            domain
                .zip(key_path)
                .is_some_and(|(domain, key_path)| format!("{domain}|{key_path}") == category_id)
        })
    }
}

fn generation_value(generation: u64) -> UiValue {
    UiValue::Int(i64::try_from(generation).unwrap_or(i64::MAX))
}

fn control_generation(generation: Option<i64>) -> u64 {
    generation
        .and_then(|generation| u64::try_from(generation).ok())
        .unwrap_or_default()
}

fn host_string(value: Option<&RetainedUiHostValue>) -> Option<&str> {
    match value {
        Some(RetainedUiHostValue::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

fn category_value(category: &crate::ui::settings::SettingsNavigationCategory) -> UiValue {
    let domain = category_domain(category.localization_domain());
    map_value([
        ("domain", UiValue::String(domain)),
        ("key_path", UiValue::String(joined(category.keys()))),
        ("label_path", UiValue::String(joined(category.labels()))),
        (
            "label",
            UiValue::String(
                category
                    .labels()
                    .last()
                    .map_or_else(String::new, ToString::to_string),
            ),
        ),
    ])
}

fn category_identity(category: &crate::ui::settings::SettingsNavigationCategory) -> String {
    format!(
        "{}|{}",
        category_domain(category.localization_domain()),
        joined(category.keys())
    )
}

fn default_category_identity(projection: &SettingsWindowProjection) -> String {
    projection
        .categories()
        .iter()
        .find(|category| category_has_direct_content(category, projection))
        .or_else(|| projection.categories().first())
        .map(category_identity)
        .unwrap_or_default()
}

fn category_has_direct_content(
    category: &crate::ui::settings::SettingsNavigationCategory,
    projection: &SettingsWindowProjection,
) -> bool {
    match category.localization_domain() {
        SettingsLocalizationDomain::BuiltIn => projection
            .settings()
            .iter()
            .any(|setting| setting.category_keys() == category.keys()),
        SettingsLocalizationDomain::Plugin(bundle_id) => {
            projection.plugin_pages().iter().any(|page| {
                page.localization_bundle_id() == bundle_id.as_ref()
                    && page.category_keys() == category.keys()
            })
        }
    }
}

fn category_domain(domain: &SettingsLocalizationDomain) -> String {
    match domain {
        SettingsLocalizationDomain::BuiltIn => "builtin".to_owned(),
        SettingsLocalizationDomain::Plugin(bundle_id) => format!("plugin:{bundle_id}"),
    }
}

fn setting_value(setting: &crate::ui::settings::LocalizedSetting) -> UiValue {
    map_value([
        ("key", UiValue::String(setting.key().to_owned())),
        ("label", UiValue::String(setting.label().to_owned())),
        (
            "description",
            UiValue::String(setting.description().to_owned()),
        ),
        (
            "category_key_path",
            UiValue::String(joined(setting.category_keys())),
        ),
        (
            "category_label_path",
            UiValue::String(joined(setting.category_labels())),
        ),
        (
            "scope",
            UiValue::String(settings_scope_name(setting.scope()).to_owned()),
        ),
        (
            "schema",
            UiValue::String(setting_schema_name(setting.schema()).to_owned()),
        ),
        ("options", setting_options(setting.schema())),
        (
            "requires_restart",
            UiValue::Bool(setting.requires_restart()),
        ),
    ])
}

fn setting_options(schema: &SettingSchema) -> UiValue {
    let SettingSchema::Enum { variants } = schema else {
        return UiValue::Array(Vec::new());
    };
    UiValue::Array(variants.iter().cloned().map(UiValue::String).collect())
}

fn settings_value_payload(values: &ResolvedSettingsBatch) -> UiValue {
    UiValue::Array(values.values().iter().map(resolved_value).collect())
}

fn resolved_value(value: &ResolvedSettingValue) -> UiValue {
    map_value([
        ("key", UiValue::String(value.key().as_str().to_owned())),
        (
            "value_text",
            UiValue::String(setting_value_text(value.value())),
        ),
        ("color_channels", setting_color_channels(value.value())),
        (
            "value_source",
            UiValue::String(setting_value_source_name(value.source()).to_owned()),
        ),
    ])
}

fn setting_color_channels(value: &SettingValue) -> UiValue {
    let SettingValue::Color(channels) = value else {
        return UiValue::Array(Vec::new());
    };
    UiValue::Array(
        channels
            .iter()
            .map(|channel| UiValue::Int(i64::from(*channel)))
            .collect(),
    )
}

fn setting_value_text(value: &SettingValue) -> String {
    match value {
        SettingValue::Bool(value) => value.to_string(),
        SettingValue::Int(value) => value.to_string(),
        SettingValue::Float(value) => value.to_string(),
        SettingValue::String(value) | SettingValue::Enum(value) => value.clone(),
        SettingValue::Chord(value) => value.to_string(),
        SettingValue::Color([red, green, blue, alpha]) => {
            format!("#{red:02X}{green:02X}{blue:02X}{alpha:02X}")
        }
        SettingValue::DesignTokens(_)
        | SettingValue::KeymapOverrides(_)
        | SettingValue::CommandPaletteMru(_) => String::new(),
    }
}

const fn setting_value_source_name(source: SettingValueSource) -> &'static str {
    match source {
        SettingValueSource::Default => "default",
        SettingValueSource::Scope(SettingsScope::User) => "user",
        SettingValueSource::Scope(SettingsScope::Project) => "project",
        SettingValueSource::Scope(SettingsScope::Session) => "session",
    }
}

fn plugin_page_value(page: &crate::core::extension::LocalizedSettingsPage) -> UiValue {
    map_value([
        ("id", UiValue::String(page.id().to_owned())),
        (
            "localization_bundle_id",
            UiValue::String(page.localization_bundle_id().to_owned()),
        ),
        ("label", UiValue::String(page.label().to_owned())),
        (
            "description",
            UiValue::String(page.description().to_owned()),
        ),
        (
            "category_key_path",
            UiValue::String(joined(page.category_keys())),
        ),
        (
            "category_label_path",
            UiValue::String(joined(page.category_labels())),
        ),
    ])
}

fn map_value<const N: usize>(entries: [(&str, UiValue); N]) -> UiValue {
    UiValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn joined(values: &[Arc<str>]) -> String {
    values
        .iter()
        .map(|value| value.as_ref())
        .collect::<Vec<_>>()
        .join("/")
}

const fn settings_scope_name(scope: SettingsScope) -> &'static str {
    match scope {
        SettingsScope::User => "user",
        SettingsScope::Project => "project",
        SettingsScope::Session => "session",
    }
}

fn setting_schema_name(schema: &SettingSchema) -> &'static str {
    match schema {
        SettingSchema::Bool => "bool",
        SettingSchema::Int { .. } => "int",
        SettingSchema::Float { .. } => "float",
        SettingSchema::String { .. } => "string",
        SettingSchema::Enum { .. } => "enum",
        SettingSchema::Color { .. } => "color",
        SettingSchema::Chord => "chord",
        SettingSchema::DesignTokens => "design_tokens",
        SettingSchema::KeymapOverrides => "keymap_overrides",
        SettingSchema::CommandPaletteMru => "command_palette_mru",
    }
}
