use thiserror::Error;

use crate::core::commands::EditorKeyChord;
use crate::core::settings::{
    SettingColorChannel, SettingNumericStepDirection, SettingSchema, SettingValue,
    SettingValueSource, SettingsError, SettingsKey, SettingsMutationError, SettingsMutationReceipt,
    SettingsPersistenceRetryReceipt, SettingsScope,
};
use crate::ui::host::EditorHostEventController;

#[derive(Debug, Error, PartialEq)]
pub(crate) enum SettingsUiMutationError {
    #[error("invalid settings key `{key}`: {message}")]
    InvalidKey { key: String, message: String },
    #[error("unknown setting `{0}`")]
    UnknownSetting(SettingsKey),
    #[error("setting `{0}` is not a boolean setting")]
    NotBoolean(SettingsKey),
    #[error("setting `{0}` is not a numeric setting")]
    NotNumeric(SettingsKey),
    #[error("setting `{0}` is not a color setting")]
    NotColor(SettingsKey),
    #[error("setting `{0}` is not a string setting")]
    NotString(SettingsKey),
    #[error("setting `{0}` is not a chord setting")]
    NotChord(SettingsKey),
    #[error("invalid key chord for setting `{key}`: {message}")]
    InvalidChord { key: SettingsKey, message: String },
    #[error("setting `{0}` is not an enum setting or `{value}` is not a declared variant")]
    InvalidEnumVariant { key: SettingsKey, value: String },
    #[error("invalid persistent settings scope `{0}`")]
    InvalidPersistenceScope(String),
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    Mutation(#[from] SettingsMutationError),
}

impl EditorHostEventController {
    pub(crate) fn toggle_bool_setting(
        &self,
        key: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        let key = parse_key(key)?;
        let authority = self.context().settings();
        let snapshot = authority.snapshot();
        let definition = snapshot
            .catalog()
            .definition(&key)
            .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
        if !matches!(definition.schema, SettingSchema::Bool) {
            return Err(SettingsUiMutationError::NotBoolean(key));
        }
        let scope = definition.scope;
        let current = authority.resolved_setting(&key)?;
        let SettingValue::Bool(current) = current.value() else {
            return Err(SettingsUiMutationError::NotBoolean(key));
        };

        self.context()
            .settings_mutations()
            .set(scope, &key, SettingValue::Bool(!*current))
            .map_err(Into::into)
    }

    pub(crate) fn reset_setting_override(
        &self,
        key: &str,
    ) -> Result<Option<SettingsMutationReceipt>, SettingsUiMutationError> {
        let key = parse_key(key)?;
        let resolved = self.context().settings().resolved_setting(&key)?;
        let SettingValueSource::Scope(scope) = resolved.source() else {
            return Ok(None);
        };
        self.context()
            .settings_mutations()
            .clear(scope, &key)
            .map(Some)
            .map_err(Into::into)
    }

    pub(crate) fn step_numeric_setting(
        &self,
        key: &str,
        direction: SettingNumericStepDirection,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        let key = parse_key(key)?;
        let authority = self.context().settings();
        let snapshot = authority.snapshot();
        let definition = snapshot
            .catalog()
            .definition(&key)
            .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
        let scope = definition.scope;
        let current = authority.resolved_setting(&key)?;
        let next = definition
            .schema
            .stepped_numeric_value(current.value(), direction)
            .ok_or_else(|| SettingsUiMutationError::NotNumeric(key.clone()))?;

        self.context()
            .settings_mutations()
            .set(scope, &key, next)
            .map_err(Into::into)
    }

    pub(crate) fn set_enum_setting(
        &self,
        key: &str,
        value: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        let key = parse_key(key)?;
        let authority = self.context().settings();
        let snapshot = authority.snapshot();
        let definition = snapshot
            .catalog()
            .definition(&key)
            .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
        let SettingSchema::Enum { variants } = &definition.schema else {
            return Err(SettingsUiMutationError::InvalidEnumVariant {
                key,
                value: value.to_owned(),
            });
        };
        if !variants.contains(value) {
            return Err(SettingsUiMutationError::InvalidEnumVariant {
                key,
                value: value.to_owned(),
            });
        }
        self.context()
            .settings_mutations()
            .set(definition.scope, &key, SettingValue::Enum(value.to_owned()))
            .map_err(Into::into)
    }

    pub(crate) fn step_color_setting(
        &self,
        key: &str,
        channel: SettingColorChannel,
        direction: SettingNumericStepDirection,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        let key = parse_key(key)?;
        let authority = self.context().settings();
        let snapshot = authority.snapshot();
        let definition = snapshot
            .catalog()
            .definition(&key)
            .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
        let scope = definition.scope;
        let current = authority.resolved_setting(&key)?;
        let next = definition
            .schema
            .stepped_color_value(current.value(), channel, direction)
            .ok_or_else(|| SettingsUiMutationError::NotColor(key.clone()))?;

        self.context()
            .settings_mutations()
            .set(scope, &key, next)
            .map_err(Into::into)
    }

    pub(crate) fn set_string_setting(
        &self,
        key: &str,
        value: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        let key = parse_key(key)?;
        let snapshot = self.context().settings().snapshot();
        let definition = snapshot
            .catalog()
            .definition(&key)
            .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
        if !matches!(definition.schema, SettingSchema::String { .. }) {
            return Err(SettingsUiMutationError::NotString(key));
        }
        self.context()
            .settings_mutations()
            .set(
                definition.scope,
                &key,
                SettingValue::String(value.to_owned()),
            )
            .map_err(Into::into)
    }

    pub(crate) fn set_chord_setting(
        &self,
        key: &str,
        value: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        let key = parse_key(key)?;
        let snapshot = self.context().settings().snapshot();
        let definition = snapshot
            .catalog()
            .definition(&key)
            .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
        if !matches!(definition.schema, SettingSchema::Chord) {
            return Err(SettingsUiMutationError::NotChord(key));
        }
        let chord = value.parse::<EditorKeyChord>().map_err(|error| {
            SettingsUiMutationError::InvalidChord {
                key: key.clone(),
                message: error.to_string(),
            }
        })?;
        self.context()
            .settings_mutations()
            .set(definition.scope, &key, SettingValue::Chord(chord))
            .map_err(Into::into)
    }

    pub(crate) fn retry_settings_persistence(
        &self,
        scope: &str,
    ) -> Result<SettingsPersistenceRetryReceipt, SettingsUiMutationError> {
        let scope = match scope {
            "project" => SettingsScope::Project,
            "user" => SettingsScope::User,
            _ => {
                return Err(SettingsUiMutationError::InvalidPersistenceScope(
                    scope.to_owned(),
                ))
            }
        };
        self.context()
            .settings_mutations()
            .retry_pending(scope)
            .map_err(Into::into)
    }
}

fn parse_key(key: &str) -> Result<SettingsKey, SettingsUiMutationError> {
    SettingsKey::parse(key).map_err(|message| SettingsUiMutationError::InvalidKey {
        key: key.to_owned(),
        message,
    })
}
