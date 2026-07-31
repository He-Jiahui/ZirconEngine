use std::ops::Deref;

use serde_json::{Map, Value};

use super::{
    application::{
        client_settings_application_plan, normalized_boolean_setting_change,
        normalized_numeric_setting_change, ClientSettingChange, ClientSettingValue,
    },
    graphics_default::{first_run_graphics_preset, GraphicsPreset, GraphicsRuntimeHints},
    registry::{BOOL_SETTINGS, NUMERIC_SETTINGS},
    state::ClientSettings,
};
use crate::preferences::PreferenceStorage;

pub const CLIENT_SETTINGS_STORAGE_KEY: &str = "woc_settings";

pub struct StoredClientSettings<S> {
    storage: S,
    settings: ClientSettings,
}

impl<S> StoredClientSettings<S>
where
    S: PreferenceStorage,
{
    pub fn new(storage: S) -> Self {
        let settings = storage
            .read(CLIENT_SETTINGS_STORAGE_KEY)
            .ok()
            .flatten()
            .map(|raw| decode_client_settings(&raw))
            .unwrap_or_default();
        Self { storage, settings }
    }

    pub fn set_numeric(&mut self, id: &str, value: f64) -> Option<f64> {
        self.set_numeric_with_application(id, value)
            .and_then(|change| match change.value {
                ClientSettingValue::Numeric(value) => Some(value),
                ClientSettingValue::Boolean(_) => None,
            })
    }

    pub fn set_boolean(&mut self, id: &str, value: bool) -> Option<bool> {
        self.set_boolean_with_application(id, value)
            .and_then(|change| match change.value {
                ClientSettingValue::Boolean(value) => Some(value),
                ClientSettingValue::Numeric(_) => None,
            })
    }

    pub fn set_numeric_with_application(
        &mut self,
        id: &str,
        value: f64,
    ) -> Option<ClientSettingChange> {
        let value = self.settings.set_numeric(id, value)?;
        self.save();
        normalized_numeric_setting_change(id, value)
    }

    pub fn set_boolean_with_application(
        &mut self,
        id: &str,
        value: bool,
    ) -> Option<ClientSettingChange> {
        let value = self.settings.set_boolean(id, value)?;
        self.save();
        normalized_boolean_setting_change(id, value)
    }

    pub fn application_plan(&self) -> Vec<ClientSettingChange> {
        client_settings_application_plan(&self.settings)
    }

    /// Persists pre-host defaults; apply `application_plan` after live subsystems are constructed.
    pub fn initialize_graphics_preset(
        &mut self,
        hints: &GraphicsRuntimeHints<'_>,
        native_mobile_runtime: bool,
    ) -> Vec<ClientSettingChange> {
        let mut changes = Vec::with_capacity(3);
        let default_already_applied = self
            .settings
            .boolean("graphicsDefaultApplied")
            .unwrap_or(false);
        if let Some(preset) = first_run_graphics_preset(default_already_applied, hints) {
            if let Some(change) =
                self.set_numeric_with_application("graphicsPreset", preset.setting_value())
            {
                changes.push(change);
            }
            if let Some(change) = self.set_boolean_with_application("graphicsDefaultApplied", true)
            {
                changes.push(change);
            }
        }
        if native_mobile_runtime
            && self.settings.numeric("graphicsPreset").unwrap_or(2.0)
                >= GraphicsPreset::Ultra.setting_value()
        {
            if let Some(change) = self.set_numeric_with_application(
                "graphicsPreset",
                GraphicsPreset::High.setting_value(),
            ) {
                changes.push(change);
            }
        }
        changes
    }

    pub fn reset(&mut self) {
        self.settings.reset();
        self.save();
    }

    pub fn into_storage(self) -> S {
        self.storage
    }

    fn save(&self) {
        let encoded = encode_client_settings(&self.settings);
        let _ = self.storage.write(CLIENT_SETTINGS_STORAGE_KEY, &encoded);
    }
}

impl<S> Deref for StoredClientSettings<S> {
    type Target = ClientSettings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

pub fn decode_client_settings(raw: &str) -> ClientSettings {
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return ClientSettings::default();
    };
    let Value::Object(values) = value else {
        return ClientSettings::default();
    };

    let mut settings = ClientSettings::default();
    for setting in NUMERIC_SETTINGS {
        let Some(Value::Number(value)) = values.get(setting.id) else {
            continue;
        };
        if let Some(value) = value.as_f64() {
            settings.set_numeric(setting.id, value);
        }
    }
    for setting in BOOL_SETTINGS {
        let Some(Value::Bool(value)) = values.get(setting.id) else {
            continue;
        };
        settings.set_boolean(setting.id, *value);
    }
    settings
}

pub fn encode_client_settings(settings: &ClientSettings) -> String {
    let mut values = Map::new();
    for setting in NUMERIC_SETTINGS {
        values.insert(
            setting.id.to_string(),
            Value::from(settings.numeric(setting.id).unwrap_or(setting.default)),
        );
    }
    for setting in BOOL_SETTINGS {
        values.insert(
            setting.id.to_string(),
            Value::Bool(settings.boolean(setting.id).unwrap_or(setting.default)),
        );
    }
    Value::Object(values).to_string()
}
