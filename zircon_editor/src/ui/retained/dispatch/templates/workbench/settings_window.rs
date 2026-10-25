use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime_interface::ui::component::UiValue;

use crate::core::settings::{
    ResolvedSettingValue, ResolvedSettingsBatch, SettingSchema, SettingValue, SettingValueSource,
    SettingsKey, SettingsMutationContext, SettingsPersistenceRetryContext, SettingsScope,
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
    values_context: WorkbenchSettingsValuesContext,
    persistence_retry_context: Option<SettingsPersistenceRetryContext>,
}

#[derive(Clone)]
pub(super) struct WorkbenchSettingsValuesContext {
    category_id: String,
    mutation: SettingsMutationContext,
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
        values: &SettingsMutationContext,
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
            settings_values: settings_value_payload(values.values()),
            plugin_pages: UiValue::Array(
                projection
                    .plugin_pages()
                    .iter()
                    .map(plugin_page_value)
                    .collect(),
            ),
            values_context: WorkbenchSettingsValuesContext {
                category_id: selected_category_id.clone(),
                mutation: values.clone(),
            },
            selected_category_id,
            settings_persistence_health_generation: health.generation(),
            settings_persistence_retry_scope: health.retry_scope_name().to_owned(),
            settings_persistence_status_text: health.status_text().to_owned(),
            persistence_retry_context: None,
        }
    }

    pub(crate) fn with_persistence_retry_context(
        mut self,
        context: Option<SettingsPersistenceRetryContext>,
    ) -> Self {
        self.persistence_retry_context = context.filter(|context| {
            settings_scope_name(context.scope()) == self.settings_persistence_retry_scope
        });
        self
    }
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn invalidate_settings_persistence_retry_context(&mut self) {
        self.settings_persistence_retry_context = None;
    }

    pub(crate) fn settings_persistence_capture_failed(&mut self, values_are_current: bool) -> bool {
        self.invalidate_settings_persistence_retry_context();
        // A busy health read must not bypass the original value/binding fence on early return.
        // A later successful health refresh alone cannot authorize a revoked value batch.
        if !values_are_current {
            self.invalidate_settings_values_context();
            return true;
        }
        false
    }

    pub(crate) fn settings_persistence_retry_context(
        &self,
        scope: &str,
    ) -> Result<SettingsPersistenceRetryContext, String> {
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN)
            || self.settings_persistence_health_needs_refresh
        {
            return Err("Settings persistence has no current display context".to_owned());
        }
        let context = self
            .settings_persistence_retry_context
            .as_ref()
            .filter(|context| settings_scope_name(context.scope()) == scope)
            .filter(|_| {
                self.control_string(
                    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                    SETTINGS_PERSISTENCE_RETRY_SCOPE,
                )
                .as_deref()
                    == Some(scope)
            })
            .ok_or_else(|| "Settings persistence has no captured retry target".to_owned())?;
        // This is the last successfully materialized target, not the event's originating frame.
        Ok(context.clone())
    }

    pub(crate) fn invalidate_settings_values_context(&mut self) {
        self.settings_values_context = None;
    }

    pub(crate) fn settings_values_context(&self) -> Option<&SettingsMutationContext> {
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN) {
            return None;
        }
        let context = self.settings_values_context.as_ref()?;
        (self
            .control_string(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, SELECTED_CATEGORY_ID)
            .as_deref()
            == Some(context.category_id.as_str()))
        .then_some(&context.mutation)
    }

    pub(crate) fn settings_mutation_context(
        &self,
        key: &str,
    ) -> Result<SettingsMutationContext, String> {
        let context = self
            .settings_values_context()
            .ok_or_else(|| "Settings values have no current display context".to_owned())?;
        let key = SettingsKey::parse(key)?;
        if context.value(&key).is_none() {
            return Err(format!("Setting `{key}` is not in the displayed category"));
        }
        // This is the latest successfully installed batch, not the event's originating frame.
        Ok(context.clone())
    }

    pub(crate) fn open_settings_window(
        &mut self,
        state: WorkbenchSettingsOpenState,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        self.invalidate_settings_values_context();
        self.invalidate_settings_persistence_retry_context();
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
        let context = state.values_context.clone();
        let retry_context = state.persistence_retry_context.clone();
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
        self.settings_values_context = Some(context);
        self.settings_persistence_retry_context = retry_context;
        self.settings_persistence_health_needs_refresh = false;
        Ok(true)
    }

    pub(crate) fn refresh_settings_window(
        &mut self,
        state: WorkbenchSettingsOpenState,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        self.invalidate_settings_values_context();
        self.invalidate_settings_persistence_retry_context();
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN) {
            return Ok(false);
        }
        let context = state.values_context.clone();
        let retry_context = state.persistence_retry_context.clone();
        self.write_settings_window_state(state)?;
        self.clear_settings_editor_open()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        self.settings_values_context = Some(context);
        self.settings_persistence_retry_context = retry_context;
        self.settings_persistence_health_needs_refresh = false;
        Ok(true)
    }

    pub(crate) fn refresh_settings_values(
        &mut self,
        values: &SettingsMutationContext,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        self.invalidate_settings_values_context();
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN) {
            return Ok(false);
        }
        // Value/category writes rebuild the whole surface; retain a retry only on full success.
        let retry_context = self.settings_persistence_retry_context.take();
        self.write_settings_values(values.values())?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        self.install_settings_values_context(values);
        self.settings_persistence_retry_context = retry_context;
        Ok(true)
    }

    pub(crate) fn refresh_settings_values_and_close_editor(
        &mut self,
        values: &SettingsMutationContext,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        self.invalidate_settings_values_context();
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN) {
            return Ok(false);
        }
        let retry_context = self.settings_persistence_retry_context.take();
        self.write_settings_values(values.values())?;
        self.clear_settings_editor_open()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        self.install_settings_values_context(values);
        self.settings_persistence_retry_context = retry_context;
        Ok(true)
    }

    pub(crate) fn settings_window_revision(&self) -> Option<WorkbenchSettingsWindowRevision> {
        self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN)
            .then(|| WorkbenchSettingsWindowRevision {
                settings_generation: self
                    .settings_values_context()
                    .map(SettingsMutationContext::generation)
                    .unwrap_or_default(),
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

    pub(crate) fn refresh_settings_persistence_health(
        &mut self,
        health: &SettingsPersistenceHealthProjection,
        context: Option<&SettingsPersistenceRetryContext>,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN) {
            self.invalidate_settings_persistence_retry_context();
            return Ok(false);
        }
        let context = context
            .filter(|context| settings_scope_name(context.scope()) == health.retry_scope_name());
        if self.settings_persistence_retry_context.as_ref() != context {
            self.invalidate_settings_persistence_retry_context();
            self.settings_persistence_health_needs_refresh = true;
        }
        // Health materialization rebuilds the whole template. A failed preparation or refresh
        // must not leave edits authorized by a batch whose presentation may be only partial.
        let values_context = self.settings_values_context.take();
        self.prepare_settings_persistence_health(health)?;
        let refreshed = self.settings_persistence_health_needs_refresh;
        if refreshed {
            // Only a successfully materialized health capture may authorize its displayed retry.
            // A failed refresh keeps the pending transaction and cannot retain an old target.
            self.refresh_prepared_state_change()?;
            self.settings_persistence_retry_context = context.cloned();
        }
        // Keep the identical displayed batch, including on a no-op; do not capture latest values.
        self.settings_values_context = values_context;
        Ok(refreshed)
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
        // Read-only notification hints cannot install a retry target. The normal host capture
        // supplies it after successful materialization, independently of setting-value refresh.
        self.invalidate_settings_persistence_retry_context();
        // Mark before the first write so a partial preparation also retries materialization.
        self.settings_persistence_health_needs_refresh = true;
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
            let matches_displayed_kind = SettingsKey::parse(setting_key)
                .ok()
                .and_then(|key| self.settings_values_context()?.value(&key))
                .is_some_and(|value| {
                    matches!(
                        (kind, value.value()),
                        (WorkbenchSettingsEditorKind::Enum, SettingValue::Enum(_))
                            | (WorkbenchSettingsEditorKind::Color, SettingValue::Color(_))
                    )
                });
            if !matches_displayed_kind {
                return Ok(false);
            }
        }
        // A partial property write or failed rebuild must not leave the previous display writable.
        // Keep the original typed capture; rebuilding does not grant a newer value generation.
        let values_context = self.settings_values_context.take();
        let retry_context = self.settings_persistence_retry_context.take();
        if open {
            self.set_settings_editor_open(setting_key, kind.as_str())?;
        } else {
            self.clear_settings_editor_open()?;
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        self.settings_values_context = values_context;
        self.settings_persistence_retry_context = retry_context;
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
        let values_context = self.settings_values_context.take();
        let retry_context = self.settings_persistence_retry_context.take();
        self.clear_settings_editor_open()?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        self.settings_values_context = values_context;
        self.settings_persistence_retry_context = retry_context;
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

    fn install_settings_values_context(&mut self, values: &SettingsMutationContext) {
        if let Some(category_id) =
            self.control_string(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, SELECTED_CATEGORY_ID)
        {
            self.settings_values_context = Some(WorkbenchSettingsValuesContext {
                category_id,
                mutation: values.clone(),
            });
        }
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
        self.invalidate_settings_values_context();
        self.invalidate_settings_persistence_retry_context();
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
        self.settings_persistence_health_needs_refresh = false;
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
        let values_context = self.settings_values_context.take();
        let retry_context = self.settings_persistence_retry_context.take();
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
        self.settings_values_context = values_context;
        self.settings_persistence_retry_context = retry_context;
        Ok(true)
    }

    pub(crate) fn select_settings_category(
        &mut self,
        control_id: &str,
        category_id: &str,
        values: &SettingsMutationContext,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        if control_id != WORKBENCH_SETTINGS_WINDOW_CONTROL_ID {
            return Ok(None);
        }
        if !self.control_bool(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, OPEN)
            || category_id.is_empty()
            || !self.settings_category_exists(category_id)
        {
            return Ok(Some(false));
        }
        self.invalidate_settings_values_context();
        let retry_context = self.settings_persistence_retry_context.take();
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
            settings_value_payload(values.values()),
        )?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        self.install_settings_values_context(values);
        self.settings_persistence_retry_context = retry_context;
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
    let value_length = values.iter().map(|value| value.len()).sum::<usize>();
    let separator_count = values.len().saturating_sub(1);
    let mut output = String::with_capacity(value_length.saturating_add(separator_count));
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push('/');
        }
        output.push_str(value.as_ref());
    }
    output
}

#[cfg(test)]
#[path = "../../workbench/tests/path_single_buffer.rs"]
mod path_single_buffer_tests;

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

#[cfg(test)]
mod guarded_preferences_tests {
    use super::*;
    use crate::core::extension::{CapabilitySet, ContributionStore};
    use crate::core::i18n::EditorI18nService;
    use crate::core::settings::{
        SettingsAuthority, SettingsMutationCoordinator, SettingsMutationDisposition,
        SettingsPersistenceLimits, SettingsPersistenceRetryDisposition, SettingsPersistenceService,
        SettingsStore, EDITOR_AUTOSAVE_INTERVAL_SECS_KEY, EDITOR_LOCALE_KEY,
        EDITOR_SHOW_PREVIEW_FEATURES_KEY,
    };
    use zircon_runtime_interface::ui::layout::UiSize;

    pub(super) fn open_state(
        mutations: &SettingsMutationCoordinator,
        category_id: &str,
    ) -> WorkbenchSettingsOpenState {
        let snapshot = mutations.authority().snapshot();
        let i18n = EditorI18nService::default();
        let projection = SettingsWindowProjection::capture(
            &snapshot,
            &ContributionStore::default().snapshot(),
            &CapabilitySet::default(),
            &i18n,
        );
        let values = mutations
            .capture_values(
                snapshot
                    .catalog()
                    .keys_for_category_subtree(category_id.strip_prefix("builtin|").unwrap()),
            )
            .unwrap();
        let health = SettingsPersistenceHealthProjection::capture(
            mutations.persistence_health_snapshot(),
            &i18n,
        );
        WorkbenchSettingsOpenState::from_projection(
            &projection,
            category_id.to_owned(),
            &values,
            &health,
        )
    }

    #[test]
    fn guarded_preferences_bridge_uses_exact_context_and_invalidates_on_close_and_source_failure() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        let category = "builtin|settings.category.editor";
        bridge
            .open_settings_window(open_state(&mutations, category))
            .unwrap();
        let displayed = bridge
            .settings_mutation_context(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY)
            .unwrap();
        // The rendered integer is presentation data and cannot replace the captured u64.
        bridge
            .mutate_control_property(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                SETTINGS_GENERATION,
                UiValue::Int(i64::MAX),
            )
            .unwrap();
        assert_eq!(
            bridge
                .settings_window_revision()
                .unwrap()
                .settings_generation,
            displayed.generation()
        );
        bridge.invalidate_settings_values_context();
        assert!(bridge
            .settings_mutation_context(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY)
            .is_err());
        let key = SettingsKey::parse(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY).unwrap();
        mutations
            .authority()
            .set(SettingsScope::Session, &key, SettingValue::Int(45))
            .unwrap();
        let snapshot = mutations.authority().snapshot();
        let refreshed = mutations
            .capture_values(
                snapshot
                    .catalog()
                    .keys_for_category_subtree("settings.category.editor"),
            )
            .unwrap();
        bridge.refresh_settings_values(&refreshed).unwrap();
        assert_eq!(
            bridge
                .settings_mutation_context(key.as_str())
                .unwrap()
                .generation(),
            refreshed.generation()
        );
        bridge.close_settings_window().unwrap();
        assert!(bridge.settings_mutation_context(key.as_str()).is_err());
        bridge
            .open_settings_window(open_state(&mutations, category))
            .unwrap();
        // Reopen intentionally exposes the latest batch; the event has no incarnation token.
        assert_eq!(
            bridge
                .settings_mutation_context(key.as_str())
                .unwrap()
                .generation(),
            refreshed.generation()
        );
    }

    #[test]
    fn preferences_health_refresh_tracks_retry_without_replacing_displayed_values() {
        let fixture =
            crate::tests::support::OwnedSourceWriteFixture::new("preferences_health_refresh")
                .unwrap();
        let authority = Arc::new(SettingsAuthority::with_defaults());
        let persistence = SettingsPersistenceService::with_limits(
            Arc::clone(&authority),
            crate::core::jobs::test_job_scheduler(),
            SettingsPersistenceLimits {
                max_entries: 1,
                max_retained_bytes: 1,
            },
        );
        let mutations = SettingsMutationCoordinator::new(
            Arc::clone(&authority),
            persistence,
            Some(SettingsStore::from_roots(fixture.root(), None)),
        );
        let key = SettingsKey::parse(EDITOR_SHOW_PREVIEW_FEATURES_KEY).unwrap();
        let receipt = mutations
            .set(SettingsScope::User, &key, SettingValue::Bool(true))
            .unwrap();
        assert!(matches!(
            receipt.disposition(),
            SettingsMutationDisposition::AppliedPendingAdmission(_)
        ));
        let i18n = EditorI18nService::default();
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge
            .open_settings_window(open_state(&mutations, "builtin|settings.category.editor"))
            .unwrap();
        bridge.update_settings_scroll_offsets(12.0, 24.0).unwrap();
        bridge
            .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
            .unwrap();
        let displayed = bridge.settings_values_context().unwrap().clone();
        let before = mutations.persistence_health_snapshot();
        let retry = mutations.retry_pending(SettingsScope::User).unwrap();
        assert_eq!(retry.file_generation(), receipt.persistence_generation());
        assert!(matches!(
            retry.disposition(),
            SettingsPersistenceRetryDisposition::PendingAdmission(_)
        ));
        let after = mutations.persistence_health_snapshot();
        assert!(after.generation() > before.generation());
        assert_eq!(authority.snapshot().generation(), displayed.generation());
        let health = SettingsPersistenceHealthProjection::capture(after, &i18n);
        assert_eq!(health.retry_scope_name(), "user");
        // The notification path may have prepared the same health without materializing it.
        assert!(bridge.prepare_settings_persistence_health(&health).unwrap());
        assert!(bridge
            .refresh_settings_persistence_health(&health, None)
            .unwrap());
        assert!(!bridge
            .refresh_settings_persistence_health(&health, None)
            .unwrap());
        assert!(bridge.settings_values_context().unwrap() == &displayed);
        assert_eq!(
            bridge
                .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
                .as_deref(),
            Some(EDITOR_LOCALE_KEY)
        );
        assert_eq!(
            bridge.control_float(WORKBENCH_SETTINGS_WINDOW_CONTROL_ID, SETTINGS_SCROLL_OFFSET),
            Some(24.0)
        );
        assert_eq!(
            bridge.control_integer(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                SETTINGS_PERSISTENCE_HEALTH_GENERATION,
            ),
            i64::try_from(health.generation()).ok()
        );
        bridge.close_settings_window().unwrap();
        mutations.retry_pending(SettingsScope::User).unwrap();
        let closed_health = SettingsPersistenceHealthProjection::capture(
            mutations.persistence_health_snapshot(),
            &i18n,
        );
        assert!(!bridge
            .refresh_settings_persistence_health(&closed_health, None)
            .unwrap());
        assert!(bridge.settings_values_context().is_none());
        mutations.shutdown().wait();
    }

    #[test]
    fn guarded_preferences_category_switch_rejects_keys_outside_the_displayed_batch() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge
            .open_settings_window(open_state(&mutations, "builtin|settings.category.editor"))
            .unwrap();
        let locale_key = SettingsKey::parse(EDITOR_LOCALE_KEY).unwrap();
        let language_values = mutations.capture_values([&locale_key]).unwrap();
        assert_eq!(
            bridge
                .select_settings_category(
                    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                    "builtin|settings.category.editor/settings.category.language",
                    &language_values
                )
                .unwrap(),
            Some(true)
        );
        assert!(bridge
            .settings_mutation_context(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY)
            .is_err());
        assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_ok());
        bridge
            .mutate_control_property(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                SELECTED_CATEGORY_ID,
                UiValue::String("builtin|settings.category.editor".to_owned()),
            )
            .unwrap();
        assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
    }

    #[test]
    fn guarded_preferences_retry_bridge_installs_captured_target_after_health_refresh_and_clears_on_close(
    ) {
        let fixture =
            crate::tests::support::OwnedSourceWriteFixture::new("preferences_retry_bridge")
                .unwrap();
        let authority = Arc::new(SettingsAuthority::with_defaults());
        let persistence = SettingsPersistenceService::with_limits(
            Arc::clone(&authority),
            crate::core::jobs::test_job_scheduler(),
            SettingsPersistenceLimits {
                max_entries: 1,
                max_retained_bytes: 1,
            },
        );
        let mutations = SettingsMutationCoordinator::new(
            authority,
            persistence,
            Some(SettingsStore::from_roots(fixture.root(), None)),
        );
        let key = SettingsKey::parse(EDITOR_SHOW_PREVIEW_FEATURES_KEY).unwrap();
        mutations
            .set(SettingsScope::User, &key, SettingValue::Bool(true))
            .unwrap();
        let (snapshot, contexts) = mutations.capture_persistence_health_with_retry().unwrap();
        let displayed = contexts
            .into_iter()
            .find(|context| context.scope() == SettingsScope::User)
            .unwrap();
        let i18n = EditorI18nService::default();
        let health = SettingsPersistenceHealthProjection::capture(snapshot, &i18n);
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        let category = "builtin|settings.category.editor";
        bridge
            .open_settings_window(
                open_state(&mutations, category)
                    .with_persistence_retry_context(Some(displayed.clone())),
            )
            .unwrap();
        assert_eq!(
            bridge.settings_persistence_retry_context("user").unwrap(),
            displayed
        );
        assert!(bridge
            .settings_persistence_retry_context("project")
            .is_err());
        assert!(!bridge
            .refresh_settings_persistence_health(&health, Some(&displayed))
            .unwrap());
        let displayed_values = bridge.settings_values_context().unwrap().clone();

        let replacement = mutations
            .set(SettingsScope::User, &key, SettingValue::Bool(false))
            .unwrap();
        let old_target = bridge.settings_persistence_retry_context("user").unwrap();
        assert_eq!(
            mutations.retry_pending_from_context(&old_target, SettingsScope::User),
            Err(crate::core::settings::SettingsMutationError::StalePersistenceRetryContext)
        );
        let (snapshot, contexts) = mutations.capture_persistence_health_with_retry().unwrap();
        let current = contexts
            .into_iter()
            .find(|context| context.scope() == SettingsScope::User)
            .unwrap();
        let health = SettingsPersistenceHealthProjection::capture(snapshot, &i18n);
        assert!(bridge.prepare_settings_persistence_health(&health).unwrap());
        // A notification hint does not install an uncaptured or unmaterialized target.
        assert!(bridge.settings_persistence_retry_context("user").is_err());
        assert!(bridge
            .refresh_settings_persistence_health(&health, Some(&current))
            .unwrap());
        assert!(bridge.settings_values_context().unwrap() == &displayed_values);
        let target = bridge.settings_persistence_retry_context("user").unwrap();
        assert_eq!(target, current);
        assert_eq!(
            mutations
                .retry_pending_from_context(&target, SettingsScope::User)
                .unwrap()
                .file_generation(),
            replacement.persistence_generation()
        );
        bridge.close_settings_window().unwrap();
        assert!(bridge.settings_persistence_retry_context("user").is_err());
        bridge
            .open_settings_window(open_state(&mutations, category))
            .unwrap();
        assert!(bridge.settings_persistence_retry_context("user").is_err());
        // A later normal capture can install the current target after reopen. Events still
        // carry no originating frame/window lease, so this does not reject an old queued event.
        mutations.shutdown().wait();
    }
}

#[cfg(test)]
mod popup_context_tests {
    use super::*;
    use crate::core::settings::{
        SettingsMutationCoordinator, EDITOR_AUTOSAVE_INTERVAL_SECS_KEY, EDITOR_LOCALE_KEY,
        EDITOR_SHOW_PREVIEW_FEATURES_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
    };
    use zircon_runtime_interface::ui::layout::UiSize;

    #[test]
    fn popup_open_requires_same_batch_matching_value_kind_without_replacing_an_open_editor() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge
            .open_settings_window(guarded_preferences_tests::open_state(
                &mutations,
                "builtin|settings.category.editor",
            ))
            .unwrap();
        let displayed = bridge.settings_values_context().unwrap().clone();
        assert!(bridge
            .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
            .unwrap());
        for (key, kind) in [
            (EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Color),
            (
                EDITOR_SHOW_PREVIEW_FEATURES_KEY,
                WorkbenchSettingsEditorKind::Enum,
            ),
            (
                EDITOR_AUTOSAVE_INTERVAL_SECS_KEY,
                WorkbenchSettingsEditorKind::Enum,
            ),
            (
                VIEWPORT_TRANSLATE_STEP_KEY,
                WorkbenchSettingsEditorKind::Enum,
            ),
            ("editor.missing.popup", WorkbenchSettingsEditorKind::Enum),
        ] {
            assert!(!bridge.toggle_settings_editor(key, kind).unwrap());
            assert_eq!(
                bridge
                    .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
                    .as_deref(),
                Some(EDITOR_LOCALE_KEY)
            );
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
        }
        mutations.shutdown().wait();
    }

    #[test]
    fn popup_close_survives_context_invalidation_but_reopen_requires_the_displayed_category() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge
            .open_settings_window(guarded_preferences_tests::open_state(
                &mutations,
                "builtin|settings.category.editor",
            ))
            .unwrap();
        assert!(bridge
            .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
            .unwrap());
        bridge.invalidate_settings_values_context();
        assert!(bridge
            .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
            .unwrap());
        assert!(bridge
            .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
            .is_none());
        assert!(!bridge
            .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
            .unwrap());
        let locale = SettingsKey::parse(EDITOR_LOCALE_KEY).unwrap();
        bridge
            .select_settings_category(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                "builtin|settings.category.editor/settings.category.language",
                &mutations.capture_values([&locale]).unwrap(),
            )
            .unwrap();
        assert!(bridge
            .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
            .unwrap());
        let autosave = SettingsKey::parse(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY).unwrap();
        bridge
            .select_settings_category(
                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                "builtin|settings.category.editor/settings.category.autosave",
                &mutations.capture_values([&autosave]).unwrap(),
            )
            .unwrap();
        assert!(bridge
            .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
            .is_none());
        assert!(!bridge
            .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
            .unwrap());
        assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
        mutations.shutdown().wait();
    }
}

#[cfg(test)]
mod popup_materialization_tests {
    use super::*;
    use crate::core::settings::{SettingsMutationCoordinator, EDITOR_LOCALE_KEY};
    use crate::ui::workbench::reference::EditorWorkbenchTemplateSurfaceError;
    use zircon_runtime_interface::ui::{layout::UiSize, tree::UiTreeError};

    #[test]
    fn popup_materialization_success_retains_the_original_displayed_context() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge
            .open_settings_window(guarded_preferences_tests::open_state(
                &mutations,
                "builtin|settings.category.editor",
            ))
            .unwrap();
        let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
        let generation = mutations.authority().snapshot().generation();
        assert!(bridge
            .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
            .unwrap());
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert!(bridge.close_settings_editor().unwrap());
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert!(bridge
            .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
            .unwrap());
        assert!(bridge.update_settings_scroll_offsets(12.0, 24.0).unwrap());
        assert!(bridge
            .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
            .is_none());
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert!(!bridge.update_settings_scroll_offsets(12.0, 24.0).unwrap());
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert!(!bridge
            .update_settings_scroll_offsets(f32::NAN, 24.0)
            .unwrap());
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert_eq!(mutations.authority().snapshot().generation(), generation);
        mutations.shutdown().wait();
    }

    #[test]
    fn popup_materialization_failure_revokes_context_until_normal_value_refresh() {
        for transition in ["open", "close", "scroll"] {
            let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                    .unwrap();
            bridge
                .open_settings_window(guarded_preferences_tests::open_state(
                    &mutations,
                    "builtin|settings.category.editor",
                ))
                .unwrap();
            let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
            if transition != "open" {
                bridge
                    .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
                    .unwrap();
            }
            // Remove one actual authored node while keeping its parent edge. Force the normal
            // layout pass to observe that missing source node after the popup property writes.
            let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
            let removed = bridge
                .template_surface
                .surface
                .tree
                .nodes
                .remove(&missing)
                .unwrap();
            for root in bridge.template_surface.surface.tree.roots.clone() {
                let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                node.dirty.layout = true;
                node.dirty.input = true;
            }
            let result = match transition {
                "open" => bridge
                    .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum),
                "close" => bridge.close_settings_editor(),
                "scroll" => bridge.update_settings_scroll_offsets(12.0, 24.0),
                _ => unreachable!(),
            };
            let error = result.unwrap_err();
            assert!(
                matches!(
                    &error,
                    BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                        EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
                    ) if *id == missing
                ),
                "{transition}: {error}"
            );
            assert!(bridge.settings_values_context().is_none());
            assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
            assert_eq!(
                mutations.authority().snapshot().generation(),
                displayed.generation()
            );
            bridge
                .template_surface
                .surface
                .tree
                .nodes
                .insert(missing, removed);
            bridge.refresh_settings_values(&displayed).unwrap();
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            mutations.shutdown().wait();
        }
    }
}

#[cfg(test)]
mod health_materialization_tests {
    use super::*;
    use crate::core::i18n::EditorI18nService;
    use crate::core::settings::{
        SettingsAuthority, SettingsMutationCoordinator, SettingsMutationDisposition,
        SettingsPersistenceLimits, SettingsPersistenceService, SettingsStore, EDITOR_LOCALE_KEY,
        EDITOR_SHOW_PREVIEW_FEATURES_KEY,
    };
    use crate::tests::support::OwnedSourceWriteFixture;
    use crate::ui::workbench::reference::EditorWorkbenchTemplateSurfaceError;
    use std::sync::Arc;
    use zircon_runtime_interface::ui::{layout::UiSize, tree::UiTreeError};

    fn pending_user_settings() -> (OwnedSourceWriteFixture, SettingsMutationCoordinator) {
        let fixture = OwnedSourceWriteFixture::new("preferences_health_materialization").unwrap();
        let authority = Arc::new(SettingsAuthority::with_defaults());
        let persistence = SettingsPersistenceService::with_limits(
            Arc::clone(&authority),
            crate::core::jobs::test_job_scheduler(),
            SettingsPersistenceLimits {
                max_entries: 1,
                max_retained_bytes: 1,
            },
        );
        let mutations = SettingsMutationCoordinator::new(
            authority,
            persistence,
            Some(SettingsStore::from_roots(fixture.root(), None)),
        );
        let key = SettingsKey::parse(EDITOR_SHOW_PREVIEW_FEATURES_KEY).unwrap();
        let receipt = mutations
            .set(SettingsScope::User, &key, SettingValue::Bool(true))
            .unwrap();
        assert!(matches!(
            receipt.disposition(),
            SettingsMutationDisposition::AppliedPendingAdmission(_)
        ));
        (fixture, mutations)
    }

    fn pending_health(
        mutations: &SettingsMutationCoordinator,
    ) -> (
        SettingsPersistenceHealthProjection,
        SettingsPersistenceRetryContext,
    ) {
        let (snapshot, contexts) = mutations.capture_persistence_health_with_retry().unwrap();
        let retry = contexts
            .into_iter()
            .find(|context| context.scope() == SettingsScope::User)
            .unwrap();
        (
            SettingsPersistenceHealthProjection::capture(snapshot, &EditorI18nService::default()),
            retry,
        )
    }

    #[test]
    fn health_materialization_success_and_noop_keep_the_identical_displayed_batch() {
        let (_fixture, mutations) = pending_user_settings();
        let (health, retry) = pending_health(&mutations);
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge
            .open_settings_window(
                guarded_preferences_tests::open_state(
                    &mutations,
                    "builtin|settings.category.editor",
                )
                .with_persistence_retry_context(Some(retry.clone())),
            )
            .unwrap();
        let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
        assert!(!bridge
            .refresh_settings_persistence_health(&health, Some(&retry))
            .unwrap());
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert_eq!(
            bridge.settings_persistence_retry_context("user").unwrap(),
            retry
        );

        mutations.retry_pending(SettingsScope::User).unwrap();
        let (health, retry) = pending_health(&mutations);
        assert!(bridge
            .refresh_settings_persistence_health(&health, Some(&retry))
            .unwrap());
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert_eq!(
            bridge.settings_persistence_retry_context("user").unwrap(),
            retry
        );
        assert!(!bridge
            .refresh_settings_persistence_health(&health, Some(&retry))
            .unwrap());
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert_eq!(
            mutations.authority().snapshot().generation(),
            displayed.generation()
        );
        mutations.shutdown().wait();
    }

    #[test]
    fn health_materialization_failure_revokes_edits_until_normal_values_refresh() {
        let (_fixture, mutations) = pending_user_settings();
        let (_, retry) = pending_health(&mutations);
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge
            .open_settings_window(
                guarded_preferences_tests::open_state(
                    &mutations,
                    "builtin|settings.category.editor",
                )
                .with_persistence_retry_context(Some(retry)),
            )
            .unwrap();
        let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
        mutations.retry_pending(SettingsScope::User).unwrap();
        let (health, retry) = pending_health(&mutations);

        // Exercise the actual template refresh error after health property preparation.
        let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
        let removed = bridge
            .template_surface
            .surface
            .tree
            .nodes
            .remove(&missing)
            .unwrap();
        for root in bridge.template_surface.surface.tree.roots.clone() {
            let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
            node.dirty.layout = true;
            node.dirty.input = true;
        }
        let error = bridge
            .refresh_settings_persistence_health(&health, Some(&retry))
            .unwrap_err();
        assert!(
            matches!(
                &error,
                BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                    EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
                ) if *id == missing
            ),
            "{error}"
        );
        assert!(bridge.settings_values_context().is_none());
        assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
        assert!(bridge.settings_persistence_retry_context("user").is_err());
        assert!(bridge.settings_persistence_health_needs_refresh);
        assert_eq!(
            mutations.authority().snapshot().generation(),
            displayed.generation()
        );
        assert_eq!(pending_health(&mutations).1, retry);

        bridge
            .template_surface
            .surface
            .tree
            .nodes
            .insert(missing, removed);
        for root in bridge.template_surface.surface.tree.roots.clone() {
            let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
            node.dirty.layout = true;
            node.dirty.input = true;
        }
        assert!(bridge
            .refresh_settings_persistence_health(&health, Some(&retry))
            .unwrap());
        assert_eq!(
            bridge.settings_persistence_retry_context("user").unwrap(),
            retry
        );
        // Successful retry presentation cannot manufacture replacement value authority.
        assert!(bridge.settings_values_context().is_none());
        bridge.refresh_settings_values(&displayed).unwrap();
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert_eq!(pending_health(&mutations).1, retry);
        assert_eq!(
            mutations.authority().snapshot().generation(),
            displayed.generation()
        );
        mutations.shutdown().wait();
    }

    mod capture_failure {
        use super::*;
        use crate::core::settings::{
            SettingChange, SettingsChangeSubscriber, SettingsMutationError, SettingsSnapshot,
            VIEWPORT_TRANSLATE_STEP_KEY,
        };
        use std::sync::{mpsc, Weak};
        use std::time::Duration;

        #[test]
        fn failed_health_capture_retires_stale_batch_without_consuming_pending_write() {
            struct PhaseProbe {
                mutations: Weak<SettingsMutationCoordinator>,
                displayed: SettingsMutationContext,
                observed: mpsc::Sender<(bool, bool)>,
            }
            impl SettingsChangeSubscriber for PhaseProbe {
                fn settings_changed(&self, _: &[SettingChange], _: &SettingsSnapshot) {
                    let mutations = self.mutations.upgrade().unwrap();
                    let health_rejected = matches!(
                        mutations.capture_persistence_health_with_retry(),
                        Err(SettingsMutationError::OperationInProgress { .. })
                    );
                    self.observed
                        .send((
                            health_rejected,
                            mutations.context_is_current(&self.displayed),
                        ))
                        .unwrap();
                }
            }

            let (_fixture, mutations) = pending_user_settings();
            let mutations = Arc::new(mutations);
            mutations
                .authority()
                .set(
                    SettingsScope::Project,
                    &SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap(),
                    SettingValue::Float(2.5),
                )
                .unwrap();
            let (health, retry) = pending_health(&mutations);
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                    .unwrap();
            bridge
                .open_settings_window(
                    guarded_preferences_tests::open_state(
                        &mutations,
                        "builtin|settings.category.editor",
                    )
                    .with_persistence_retry_context(Some(retry.clone())),
                )
                .unwrap();
            let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
            let (sender, receiver) = mpsc::channel();
            mutations
                .authority()
                .configure_change_subscriber(Arc::new(PhaseProbe {
                    mutations: Arc::downgrade(&mutations),
                    displayed: displayed.clone(),
                    observed: sender,
                }));

            // Observe the actual coordinator phase while its project clear publishes a change.
            mutations.clear_project().unwrap();
            let (health_rejected, values_are_current) =
                receiver.recv_timeout(Duration::from_secs(2)).unwrap();
            assert!(health_rejected);
            assert!(!values_are_current);
            assert!(bridge.settings_persistence_capture_failed(values_are_current));
            assert!(bridge.settings_values_context().is_none());
            assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            assert_eq!(pending_health(&mutations).1, retry);

            // Recovered health may restore retry, but it cannot reinstall the old value batch.
            bridge
                .refresh_settings_persistence_health(&health, Some(&retry))
                .unwrap();
            assert!(bridge.settings_values_context().is_none());
            assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
            let fresh = guarded_preferences_tests::open_state(
                &mutations,
                "builtin|settings.category.editor",
            )
            .values_context
            .mutation;
            assert!(fresh.generation() > displayed.generation());
            bridge.refresh_settings_values(&fresh).unwrap();
            assert_eq!(bridge.settings_values_context(), Some(&fresh));
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert_eq!(pending_health(&mutations).1, retry);
            mutations.shutdown().wait();
        }

        #[test]
        fn failed_health_capture_keeps_current_values_and_requires_retry_recapture() {
            let (_fixture, mutations) = pending_user_settings();
            let (health, retry) = pending_health(&mutations);
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                    .unwrap();
            bridge
                .open_settings_window(
                    guarded_preferences_tests::open_state(
                        &mutations,
                        "builtin|settings.category.editor",
                    )
                    .with_persistence_retry_context(Some(retry.clone())),
                )
                .unwrap();
            let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
            assert!(mutations.context_is_current(&displayed));
            assert!(!bridge.settings_persistence_capture_failed(true));
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            bridge
                .refresh_settings_persistence_health(&health, Some(&retry))
                .unwrap();
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert_eq!(pending_health(&mutations).1, retry);
            assert_eq!(
                mutations.authority().snapshot().generation(),
                displayed.generation()
            );
            mutations.shutdown().wait();
        }
    }

    mod value_category_state_change {
        use super::*;

        const EDITOR_CATEGORY: &str = "builtin|settings.category.editor";
        const LANGUAGE_CATEGORY: &str =
            "builtin|settings.category.editor/settings.category.language";

        fn category(name: &str) -> &'static str {
            if name == "category" {
                LANGUAGE_CATEGORY
            } else {
                EDITOR_CATEGORY
            }
        }

        fn transition(
            bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
            name: &str,
            values: &SettingsMutationContext,
        ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
            match name {
                "values" => bridge.refresh_settings_values(values),
                "values_close" => bridge.refresh_settings_values_and_close_editor(values),
                "category" => bridge
                    .select_settings_category(
                        WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                        LANGUAGE_CATEGORY,
                        values,
                    )
                    .map(|handled| handled.expect("settings category must be handled")),
                _ => unreachable!("unknown test transition"),
            }
        }

        fn mark_layout_dirty(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge) {
            for root in bridge.template_surface.surface.tree.roots.clone() {
                let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                node.dirty.layout = true;
                node.dirty.input = true;
            }
        }

        #[test]
        fn value_and_category_success_preserves_only_supplied_values_and_original_retry() {
            for name in ["values", "values_close", "category"] {
                for with_retry in [true, false] {
                    let (_fixture, mutations) = pending_user_settings();
                    let (_, retry) = pending_health(&mutations);
                    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
                        1280.0, 800.0,
                    ))
                    .unwrap();
                    bridge
                        .open_settings_window(
                            guarded_preferences_tests::open_state(&mutations, EDITOR_CATEGORY)
                                .with_persistence_retry_context(Some(retry.clone())),
                        )
                        .unwrap();
                    assert!(bridge
                        .toggle_settings_editor(
                            EDITOR_LOCALE_KEY,
                            WorkbenchSettingsEditorKind::Enum
                        )
                        .unwrap());
                    let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
                    let key = SettingsKey::parse(EDITOR_SHOW_PREVIEW_FEATURES_KEY).unwrap();
                    assert!(mutations
                        .set(SettingsScope::Session, &key, SettingValue::Bool(false))
                        .unwrap()
                        .changed());
                    let supplied =
                        guarded_preferences_tests::open_state(&mutations, category(name))
                            .values_context
                            .mutation;
                    assert!(mutations
                        .set(SettingsScope::Session, &key, SettingValue::Bool(true))
                        .unwrap()
                        .changed());
                    let current_generation = mutations.authority().snapshot().generation();
                    assert_ne!(supplied.generation(), displayed.generation());
                    assert_ne!(supplied.generation(), current_generation);
                    assert!(!mutations.context_is_current(&supplied));
                    if !with_retry {
                        // Receipt callers already revoke retry before preparing health.
                        bridge.invalidate_settings_persistence_retry_context();
                    }
                    let original_retry = bridge.settings_persistence_retry_context.clone();
                    assert_eq!(
                        bridge
                            .select_settings_category("other", LANGUAGE_CATEGORY, &supplied)
                            .unwrap(),
                        None
                    );
                    for invalid in ["", "builtin|missing"] {
                        assert_eq!(
                            bridge
                                .select_settings_category(
                                    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                                    invalid,
                                    &supplied,
                                )
                                .unwrap(),
                            Some(false)
                        );
                    }
                    assert_eq!(bridge.settings_values_context(), Some(&displayed));
                    assert_eq!(bridge.settings_persistence_retry_context, original_retry);

                    assert!(transition(&mut bridge, name, &supplied).unwrap());
                    assert_eq!(bridge.settings_values_context(), Some(&supplied));
                    assert_eq!(bridge.settings_persistence_retry_context, original_retry);
                    if with_retry {
                        assert_eq!(
                            bridge.settings_persistence_retry_context("user").unwrap(),
                            retry
                        );
                    } else {
                        assert!(bridge.settings_persistence_retry_context("user").is_err());
                    }
                    if name == "values" {
                        assert_eq!(
                            bridge.settings_editor_open_key(WorkbenchSettingsEditorKind::Enum),
                            Some(EDITOR_LOCALE_KEY.to_owned())
                        );
                    } else {
                        assert!(bridge
                            .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
                            .is_none());
                    }
                    assert_eq!(pending_health(&mutations).1, retry);
                    assert_eq!(
                        mutations.authority().snapshot().generation(),
                        current_generation
                    );
                    bridge.close_settings_window().unwrap();
                    assert!(!transition(&mut bridge, name, &supplied).unwrap());
                    assert!(bridge.settings_values_context().is_none());
                    assert!(bridge.settings_persistence_retry_context.is_none());
                    mutations.shutdown().wait();
                }
            }
        }

        #[test]
        fn value_and_category_rebuild_failure_revokes_both_contexts_until_recaptured() {
            for name in ["values", "values_close", "category"] {
                let (_fixture, mutations) = pending_user_settings();
                let (health, retry) = pending_health(&mutations);
                let mut bridge =
                    BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                        .unwrap();
                bridge
                    .open_settings_window(
                        guarded_preferences_tests::open_state(&mutations, EDITOR_CATEGORY)
                            .with_persistence_retry_context(Some(retry.clone())),
                    )
                    .unwrap();
                assert!(bridge
                    .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
                    .unwrap());
                let supplied = guarded_preferences_tests::open_state(&mutations, category(name))
                    .values_context
                    .mutation;
                let original_generation = mutations.authority().snapshot().generation();
                let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
                let removed = bridge
                    .template_surface
                    .surface
                    .tree
                    .nodes
                    .remove(&missing)
                    .unwrap();
                mark_layout_dirty(&mut bridge);
                let error = transition(&mut bridge, name, &supplied).unwrap_err();
                assert!(
                    matches!(
                        &error,
                        BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                            EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
                        ) if *id == missing
                    ),
                    "{name}: {error}"
                );
                assert!(bridge.settings_values_context().is_none());
                assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
                assert!(bridge.settings_persistence_retry_context.is_none());
                assert!(bridge.settings_persistence_retry_context("user").is_err());
                assert_eq!(pending_health(&mutations).1, retry);
                assert_eq!(
                    mutations.authority().snapshot().generation(),
                    original_generation
                );

                bridge
                    .template_surface
                    .surface
                    .tree
                    .nodes
                    .insert(missing, removed);
                mark_layout_dirty(&mut bridge);
                bridge.refresh_prepared_state_change().unwrap();
                assert!(bridge.settings_values_context().is_none());
                assert!(bridge.settings_persistence_retry_context("user").is_err());
                assert!(bridge
                    .refresh_settings_persistence_health(&health, Some(&retry))
                    .unwrap());
                assert_eq!(
                    bridge.settings_persistence_retry_context("user").unwrap(),
                    retry
                );
                assert!(bridge.settings_values_context().is_none());
                bridge.refresh_settings_values(&supplied).unwrap();
                assert_eq!(bridge.settings_values_context(), Some(&supplied));
                assert_eq!(
                    bridge.settings_persistence_retry_context("user").unwrap(),
                    retry
                );
                assert_eq!(pending_health(&mutations).1, retry);
                assert_eq!(
                    mutations.authority().snapshot().generation(),
                    original_generation
                );
                mutations.shutdown().wait();
            }
        }
    }

    mod popup_scroll_state_change {
        use super::*;

        fn prepare_transition(
            bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
            transition: &str,
        ) {
            if matches!(transition, "toggle_close" | "close") {
                assert!(bridge
                    .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
                    .unwrap());
            }
        }

        fn transition(
            bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
            name: &str,
        ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
            match name {
                "toggle_open" | "toggle_close" => bridge
                    .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum),
                "close" => bridge.close_settings_editor(),
                "scroll" => bridge.update_settings_scroll_offsets(12.0, 24.0),
                _ => unreachable!("unknown test transition"),
            }
        }

        #[test]
        fn successful_popup_and_scroll_rebuilds_keep_only_the_original_contexts() {
            for name in ["toggle_open", "toggle_close", "close", "scroll"] {
                let (_fixture, mutations) = pending_user_settings();
                let (_, retry) = pending_health(&mutations);
                let mut bridge =
                    BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                        .unwrap();
                bridge
                    .open_settings_window(
                        guarded_preferences_tests::open_state(
                            &mutations,
                            "builtin|settings.category.editor",
                        )
                        .with_persistence_retry_context(Some(retry.clone())),
                    )
                    .unwrap();
                prepare_transition(&mut bridge, name);
                let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
                assert!(!bridge.update_settings_scroll_offsets(0.0, 0.0).unwrap());
                assert_eq!(bridge.settings_values_context(), Some(&displayed));
                assert_eq!(
                    bridge.settings_persistence_retry_context("user").unwrap(),
                    retry
                );
                assert!(transition(&mut bridge, name).unwrap());
                if name == "scroll" {
                    assert!(!bridge.update_settings_scroll_offsets(12.0, 24.0).unwrap());
                } else if name == "close" {
                    assert!(!bridge.close_settings_editor().unwrap());
                }
                assert_eq!(bridge.settings_values_context(), Some(&displayed));
                assert_eq!(
                    bridge.settings_persistence_retry_context("user").unwrap(),
                    retry
                );
                assert_eq!(pending_health(&mutations).1, retry);
                assert_eq!(
                    mutations.authority().snapshot().generation(),
                    displayed.generation()
                );
                mutations.shutdown().wait();
            }
        }

        #[test]
        fn failed_popup_and_scroll_rebuilds_revoke_both_contexts_until_recaptured() {
            for name in ["toggle_open", "toggle_close", "close", "scroll"] {
                let (_fixture, mutations) = pending_user_settings();
                let (health, retry) = pending_health(&mutations);
                let mut bridge =
                    BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                        .unwrap();
                bridge
                    .open_settings_window(
                        guarded_preferences_tests::open_state(
                            &mutations,
                            "builtin|settings.category.editor",
                        )
                        .with_persistence_retry_context(Some(retry.clone())),
                    )
                    .unwrap();
                prepare_transition(&mut bridge, name);
                let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
                assert_eq!(
                    bridge.settings_persistence_retry_context("user").unwrap(),
                    retry
                );
                assert!(!bridge.settings_persistence_health_needs_refresh);
                let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
                let removed = bridge
                    .template_surface
                    .surface
                    .tree
                    .nodes
                    .remove(&missing)
                    .unwrap();
                for root in bridge.template_surface.surface.tree.roots.clone() {
                    let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                    node.dirty.layout = true;
                    node.dirty.input = true;
                }
                let error = transition(&mut bridge, name).unwrap_err();
                assert!(
                    matches!(
                        &error,
                        BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                            EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
                        ) if *id == missing
                    ),
                    "{name}: {error}"
                );
                assert!(bridge.settings_values_context().is_none());
                assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
                assert!(bridge.settings_persistence_retry_context.is_none());
                assert!(bridge.settings_persistence_retry_context("user").is_err());
                assert_eq!(pending_health(&mutations).1, retry);
                assert_eq!(
                    mutations.authority().snapshot().generation(),
                    displayed.generation()
                );

                bridge
                    .template_surface
                    .surface
                    .tree
                    .nodes
                    .insert(missing, removed);
                for root in bridge.template_surface.surface.tree.roots.clone() {
                    let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                    node.dirty.layout = true;
                    node.dirty.input = true;
                }
                bridge.refresh_prepared_state_change().unwrap();
                assert!(bridge.settings_values_context().is_none());
                assert!(bridge.settings_persistence_retry_context("user").is_err());
                assert!(bridge
                    .refresh_settings_persistence_health(&health, Some(&retry))
                    .unwrap());
                assert_eq!(
                    bridge.settings_persistence_retry_context("user").unwrap(),
                    retry
                );
                assert!(bridge.settings_values_context().is_none());
                bridge.refresh_settings_values(&displayed).unwrap();
                assert_eq!(bridge.settings_values_context(), Some(&displayed));
                assert_eq!(pending_health(&mutations).1, retry);
                assert_eq!(
                    mutations.authority().snapshot().generation(),
                    displayed.generation()
                );
                mutations.shutdown().wait();
            }
        }
    }

    mod prepared_state_change {
        use super::*;

        #[test]
        fn successful_prepared_rebuild_preserves_only_the_original_settings_contexts() {
            let (_fixture, mutations) = pending_user_settings();
            let (health, retry) = pending_health(&mutations);
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                    .unwrap();
            bridge
                .open_settings_window(
                    guarded_preferences_tests::open_state(
                        &mutations,
                        "builtin|settings.category.editor",
                    )
                    .with_persistence_retry_context(Some(retry.clone())),
                )
                .unwrap();
            let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
            bridge.refresh_prepared_state_change().unwrap();
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert!(!bridge
                .refresh_settings_persistence_health(&health, Some(&retry))
                .unwrap());

            mutations.retry_pending(SettingsScope::User).unwrap();
            let (health, retry) = pending_health(&mutations);
            // Use the production notification order: prepare, then the shared rebuild.
            assert!(bridge.prepare_settings_persistence_health(&health).unwrap());
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            bridge.refresh_prepared_state_change().unwrap();
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            // A prepared notification cannot manufacture a new retry target.
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            assert!(bridge
                .refresh_settings_persistence_health(&health, Some(&retry))
                .unwrap());
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert_eq!(
                mutations.authority().snapshot().generation(),
                displayed.generation()
            );
            mutations.shutdown().wait();
        }

        #[test]
        fn failed_prepared_rebuild_revokes_settings_contexts_until_explicit_refresh() {
            for prepare_health in [false, true] {
                let (_fixture, mutations) = pending_user_settings();
                let (_, retry) = pending_health(&mutations);
                let mut bridge =
                    BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                        .unwrap();
                bridge
                    .open_settings_window(
                        guarded_preferences_tests::open_state(
                            &mutations,
                            "builtin|settings.category.editor",
                        )
                        .with_persistence_retry_context(Some(retry.clone())),
                    )
                    .unwrap();
                let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
                if prepare_health {
                    mutations.retry_pending(SettingsScope::User).unwrap();
                    let (health, _) = pending_health(&mutations);
                    assert!(bridge.prepare_settings_persistence_health(&health).unwrap());
                }
                let (health, retry) = pending_health(&mutations);
                let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
                let removed = bridge
                    .template_surface
                    .surface
                    .tree
                    .nodes
                    .remove(&missing)
                    .unwrap();
                for root in bridge.template_surface.surface.tree.roots.clone() {
                    let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                    node.dirty.layout = true;
                    node.dirty.input = true;
                }
                let error = bridge.refresh_prepared_state_change().unwrap_err();
                assert!(
                    matches!(
                        &error,
                        BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                            EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
                        ) if *id == missing
                    ),
                    "prepare_health={prepare_health}: {error}"
                );
                assert!(bridge.settings_values_context().is_none());
                assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
                assert!(bridge.settings_persistence_retry_context("user").is_err());
                assert_eq!(pending_health(&mutations).1, retry);
                assert_eq!(
                    mutations.authority().snapshot().generation(),
                    displayed.generation()
                );

                bridge
                    .template_surface
                    .surface
                    .tree
                    .nodes
                    .insert(missing, removed);
                for root in bridge.template_surface.surface.tree.roots.clone() {
                    let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                    node.dirty.layout = true;
                    node.dirty.input = true;
                }
                bridge.refresh_prepared_state_change().unwrap();
                assert!(bridge.settings_values_context().is_none());
                assert!(bridge.settings_persistence_retry_context("user").is_err());
                assert!(bridge
                    .refresh_settings_persistence_health(&health, Some(&retry))
                    .unwrap());
                assert!(bridge.settings_values_context().is_none());
                assert_eq!(
                    bridge.settings_persistence_retry_context("user").unwrap(),
                    retry
                );
                bridge.refresh_settings_values(&displayed).unwrap();
                assert_eq!(bridge.settings_values_context(), Some(&displayed));
                assert_eq!(pending_health(&mutations).1, retry);
                assert_eq!(
                    mutations.authority().snapshot().generation(),
                    displayed.generation()
                );
                mutations.shutdown().wait();
            }
        }
    }
}
