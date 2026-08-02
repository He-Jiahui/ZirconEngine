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
use crate::preferences::{read_preference_text, submit_preference_text, PreferenceRead};
use zircon_runtime::core::framework::platform::{PreferenceMutationSubmission, PreferenceStorage};

pub const CLIENT_SETTINGS_STORAGE_KEY: &str = "woc_settings";

pub struct StoredClientSettings<S> {
    storage: S,
    settings: ClientSettings,
    last_persistence_submission: Option<PreferenceMutationSubmission>,
}

impl<S> StoredClientSettings<S>
where
    S: AsRef<dyn PreferenceStorage>,
{
    pub fn new(storage: S) -> Self {
        let settings = read_settings(&storage).into_ready().unwrap_or_default();
        Self {
            storage,
            settings,
            last_persistence_submission: None,
        }
    }

    pub fn refresh_from_storage(&mut self) -> bool {
        let Some(settings) = read_settings(&self.storage).into_ready() else {
            return false;
        };
        self.settings = settings;
        true
    }

    pub fn take_persistence_submission(&mut self) -> Option<PreferenceMutationSubmission> {
        self.last_persistence_submission.take()
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

    fn save(&mut self) {
        let encoded = encode_client_settings(&self.settings);
        self.last_persistence_submission =
            submit_preference_text(self.storage.as_ref(), CLIENT_SETTINGS_STORAGE_KEY, &encoded);
    }
}

impl<S> Deref for StoredClientSettings<S> {
    type Target = ClientSettings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

fn read_settings<S>(storage: &S) -> PreferenceRead<ClientSettings>
where
    S: AsRef<dyn PreferenceStorage>,
{
    read_preference_text(storage.as_ref(), CLIENT_SETTINGS_STORAGE_KEY).map(|raw| {
        raw.as_deref()
            .map(decode_client_settings)
            .unwrap_or_default()
    })
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
