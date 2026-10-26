use thiserror::Error;

use crate::core::commands::EditorKeyChord;
use crate::core::settings::{
    SettingColorChannel, SettingNumericStepDirection, SettingSchema, SettingValue, SettingsError,
    SettingsKey, SettingsMutationContext, SettingsMutationCoordinator, SettingsMutationError,
    SettingsMutationReceipt, SettingsPersistenceRetryContext, SettingsPersistenceRetryReceipt,
    SettingsScope,
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
    #[error("invalid numeric value for setting `{key}`: {message}")]
    InvalidNumericText { key: SettingsKey, message: String },
    #[error("setting `{0}` is not a color setting")]
    NotColor(SettingsKey),
    #[error("setting `{0}` is not a string setting")]
    NotString(SettingsKey),
    #[error("setting `{0}` is not a chord setting")]
    NotChord(SettingsKey),
    #[error("invalid key chord for setting `{key}`: {message}")]
    InvalidChord { key: SettingsKey, message: String },
    #[error("setting `{key}` is not an enum setting or `{value}` is not a declared variant")]
    InvalidEnumVariant { key: SettingsKey, value: String },
    #[error("invalid persistent settings scope `{0}`")]
    InvalidPersistenceScope(String),
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    Mutation(#[from] SettingsMutationError),
}

impl EditorHostEventController {
    pub(crate) fn set_setting_in_scope(
        &self,
        key: &str,
        scope: SettingsScope,
        value: SettingValue,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        let key = parse_key(key)?;
        set_override(self.context().settings_mutations(), &key, scope, value)
    }

    pub(crate) fn toggle_bool_setting(
        &self,
        context: &SettingsMutationContext,
        key: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        edit_setting_from_context(
            self.context().settings_mutations(),
            context,
            key,
            CapturedSettingsEdit::ToggleBool,
        )
    }

    pub(crate) fn reset_setting_override(
        &self,
        key: &str,
    ) -> Result<Option<SettingsMutationReceipt>, SettingsUiMutationError> {
        reset_override(self.context().settings_mutations(), key, None)
    }

    pub(crate) fn reset_setting_override_in_scope(
        &self,
        key: &str,
        scope: SettingsScope,
    ) -> Result<Option<SettingsMutationReceipt>, SettingsUiMutationError> {
        reset_override(self.context().settings_mutations(), key, Some(scope))
    }

    pub(crate) fn reset_setting_override_from_context(
        &self,
        context: &SettingsMutationContext,
        key: &str,
    ) -> Result<Option<SettingsMutationReceipt>, SettingsUiMutationError> {
        reset_override_from_context(self.context().settings_mutations(), context, key)
    }

    pub(crate) fn step_numeric_setting(
        &self,
        context: &SettingsMutationContext,
        key: &str,
        direction: SettingNumericStepDirection,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        edit_setting_from_context(
            self.context().settings_mutations(),
            context,
            key,
            CapturedSettingsEdit::NumericStep(direction),
        )
    }

    pub(crate) fn set_numeric_setting(
        &self,
        context: &SettingsMutationContext,
        key: &str,
        value: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        edit_setting_from_context(
            self.context().settings_mutations(),
            context,
            key,
            CapturedSettingsEdit::NumericText(value),
        )
    }

    pub(crate) fn set_numeric_setting_in_scope(
        &self,
        key: &str,
        scope: SettingsScope,
        value: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        set_numeric_override(self.context().settings_mutations(), key, scope, value)
    }

    pub(crate) fn set_enum_setting(
        &self,
        context: &SettingsMutationContext,
        key: &str,
        value: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        edit_setting_from_context(
            self.context().settings_mutations(),
            context,
            key,
            CapturedSettingsEdit::Enum(value),
        )
    }

    pub(crate) fn step_color_setting(
        &self,
        context: &SettingsMutationContext,
        key: &str,
        channel: SettingColorChannel,
        direction: SettingNumericStepDirection,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        edit_setting_from_context(
            self.context().settings_mutations(),
            context,
            key,
            CapturedSettingsEdit::ColorStep(channel, direction),
        )
    }

    pub(crate) fn set_string_setting(
        &self,
        context: &SettingsMutationContext,
        key: &str,
        value: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        set_string_override_from_context(self.context().settings_mutations(), context, key, value)
    }

    pub(crate) fn set_chord_setting(
        &self,
        context: &SettingsMutationContext,
        key: &str,
        value: &str,
    ) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
        edit_setting_from_context(
            self.context().settings_mutations(),
            context,
            key,
            CapturedSettingsEdit::Chord(value),
        )
    }

    pub(crate) fn retry_settings_persistence(
        &self,
        context: &SettingsPersistenceRetryContext,
        scope: &str,
    ) -> Result<SettingsPersistenceRetryReceipt, SettingsUiMutationError> {
        let scope = match scope {
            "project" => SettingsScope::Project,
            "user" => SettingsScope::User,
            _ => {
                return Err(SettingsUiMutationError::InvalidPersistenceScope(
                    scope.to_owned(),
                ));
            }
        };
        self.context()
            .settings_mutations()
            .retry_pending_from_context(context, scope)
            .map_err(Into::into)
    }
}

#[derive(Clone, Copy)]
enum CapturedSettingsEdit<'a> {
    ToggleBool,
    NumericStep(SettingNumericStepDirection),
    NumericText(&'a str),
    ColorStep(SettingColorChannel, SettingNumericStepDirection),
    Enum(&'a str),
    Chord(&'a str),
}

fn edit_setting_from_context(
    mutations: &SettingsMutationCoordinator,
    context: &SettingsMutationContext,
    key: &str,
    edit: CapturedSettingsEdit<'_>,
) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
    let key = parse_key(key)?;
    let snapshot = mutations.authority().snapshot();
    let definition = snapshot
        .catalog()
        .definition(&key)
        .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
    let displayed = context
        .value(&key)
        .ok_or_else(|| SettingsMutationError::KeyNotCaptured(key.clone()))?;
    // Derive relative edits from the displayed value. The latest snapshot supplies schema only;
    // the original coordinator/authority still checks provenance, binding and generation atomically.
    let value = match edit {
        CapturedSettingsEdit::ToggleBool => match (&definition.schema, displayed.value()) {
            (SettingSchema::Bool, SettingValue::Bool(value)) => SettingValue::Bool(!*value),
            _ => return Err(SettingsUiMutationError::NotBoolean(key)),
        },
        CapturedSettingsEdit::NumericText(value) => {
            parse_numeric_setting_value(&definition.schema, &key, value)?
        }
        CapturedSettingsEdit::NumericStep(direction) => definition
            .schema
            .stepped_numeric_value(displayed.value(), direction)
            .ok_or_else(|| SettingsUiMutationError::NotNumeric(key.clone()))?,
        CapturedSettingsEdit::ColorStep(channel, direction) => definition
            .schema
            .stepped_color_value(displayed.value(), channel, direction)
            .ok_or_else(|| SettingsUiMutationError::NotColor(key.clone()))?,
        CapturedSettingsEdit::Enum(value) => {
            if !matches!(&definition.schema, SettingSchema::Enum { variants } if variants.contains(value))
            {
                return Err(SettingsUiMutationError::InvalidEnumVariant {
                    key,
                    value: value.to_owned(),
                });
            }
            SettingValue::Enum(value.to_owned())
        }
        CapturedSettingsEdit::Chord(value) => {
            if !matches!(definition.schema, SettingSchema::Chord) {
                return Err(SettingsUiMutationError::NotChord(key));
            }
            let chord = value.parse::<EditorKeyChord>().map_err(|error| {
                SettingsUiMutationError::InvalidChord {
                    key: key.clone(),
                    message: error.to_string(),
                }
            })?;
            SettingValue::Chord(chord)
        }
    };
    mutations
        .set_from_context(context, definition.scope, &key, value)
        .map_err(Into::into)
}

fn set_string_override_from_context(
    mutations: &SettingsMutationCoordinator,
    context: &SettingsMutationContext,
    key: &str,
    value: &str,
) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
    let key = parse_key(key)?;
    let snapshot = mutations.authority().snapshot();
    let definition = snapshot
        .catalog()
        .definition(&key)
        .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
    if !matches!(definition.schema, SettingSchema::String { .. }) {
        return Err(SettingsUiMutationError::NotString(key));
    }
    // The snapshot supplies schema, not evidence of the displayed value generation.
    mutations
        .set_from_context(
            context,
            definition.scope,
            &key,
            SettingValue::String(value.to_owned()),
        )
        .map_err(Into::into)
}

fn reset_override_from_context(
    mutations: &SettingsMutationCoordinator,
    context: &SettingsMutationContext,
    key: &str,
) -> Result<Option<SettingsMutationReceipt>, SettingsUiMutationError> {
    let key = parse_key(key)?;
    let snapshot = mutations.authority().snapshot();
    let definition = snapshot
        .catalog()
        .definition(&key)
        .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
    // Keep Preferences' existing definition scope; other pages still need explicit callers.
    mutations
        .clear_from_context(context, definition.scope, &key)
        .map_err(Into::into)
}

fn parse_key(key: &str) -> Result<SettingsKey, SettingsUiMutationError> {
    SettingsKey::parse(key).map_err(|message| SettingsUiMutationError::InvalidKey {
        key: key.to_owned(),
        message,
    })
}

fn set_numeric_override(
    mutations: &SettingsMutationCoordinator,
    key: &str,
    scope: SettingsScope,
    value: &str,
) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
    let key = parse_key(key)?;
    let snapshot = mutations.authority().snapshot();
    let definition = snapshot
        .catalog()
        .definition(&key)
        .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
    let value = parse_numeric_setting_value(&definition.schema, &key, value)?;
    set_override(mutations, &key, scope, value)
}

fn parse_numeric_setting_value(
    schema: &SettingSchema,
    key: &SettingsKey,
    value: &str,
) -> Result<SettingValue, SettingsUiMutationError> {
    let parsed = match schema {
        SettingSchema::Int { .. } => value
            .trim()
            .parse::<i64>()
            .map(SettingValue::Int)
            .map_err(|error| error.to_string()),
        SettingSchema::Float { .. } => value
            .trim()
            .parse::<f64>()
            .map(SettingValue::Float)
            .map_err(|error| error.to_string()),
        _ => return Err(SettingsUiMutationError::NotNumeric(key.clone())),
    };
    parsed.map_err(|message| SettingsUiMutationError::InvalidNumericText {
        key: key.clone(),
        message,
    })
}

fn set_override(
    mutations: &SettingsMutationCoordinator,
    key: &SettingsKey,
    scope: SettingsScope,
    value: SettingValue,
) -> Result<SettingsMutationReceipt, SettingsUiMutationError> {
    let snapshot = mutations.authority().snapshot();
    let definition = snapshot
        .catalog()
        .definition(key)
        .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
    if !definition.scope.allows_write(scope) {
        return Err(SettingsError::ScopeNotAllowed {
            key: key.as_str().to_owned(),
            requested: scope,
            defined: definition.scope,
        }
        .into());
    }
    definition
        .schema
        .validate(&value)
        .map_err(|reason| SettingsError::InvalidValue {
            key: key.as_str().to_owned(),
            reason,
        })?;

    // Keep typed page edits on the existing authority and persistence transaction path.
    mutations.set(scope, key, value).map_err(Into::into)
}

fn reset_override(
    mutations: &SettingsMutationCoordinator,
    key: &str,
    requested_scope: Option<SettingsScope>,
) -> Result<Option<SettingsMutationReceipt>, SettingsUiMutationError> {
    let key = parse_key(key)?;
    let snapshot = mutations.authority().snapshot();
    let definition = snapshot
        .catalog()
        .definition(&key)
        .ok_or_else(|| SettingsUiMutationError::UnknownSetting(key.clone()))?;
    let scope = requested_scope.unwrap_or(definition.scope);
    if !definition.scope.allows_write(scope) {
        return Err(SettingsError::ScopeNotAllowed {
            key: key.as_str().to_owned(),
            requested: scope,
            defined: definition.scope,
        }
        .into());
    }

    // Read this key's actual target layer, including overrides hidden by Session.
    // An absent layer has nothing to reset and needs no persistence admission.
    let values = mutations
        .authority()
        .resolved_settings(std::slice::from_ref(&key))?;
    if !values
        .values()
        .iter()
        .any(|value| value.override_in_scope(scope).is_some())
    {
        return Ok(None);
    }

    // A page owns its edit layer even when another layer supplies the effective value.
    // Clearing through the coordinator also handles an override hidden by Session.
    let receipt = mutations.clear(scope, &key)?;
    Ok(receipt.changed().then_some(receipt))
}

#[cfg(test)]
#[path = "tests/prefs.rs"]
mod guarded_preferences_tests;

#[cfg(test)]
#[path = "tests/scope.rs"]
mod settings_scope_mutation_tests;

#[cfg(test)]
#[path = "tests/numeric.rs"]
mod guarded_numeric_text_tests;
