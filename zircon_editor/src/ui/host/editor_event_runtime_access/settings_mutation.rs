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
mod guarded_preferences_tests {
    use super::*;
    use crate::core::settings::{
        settings_registry_with_defaults, SettingDefinition, SettingsAuthority,
        SettingsPersistenceService, SettingsPresentation,
    };
    use std::sync::Arc;

    #[test]
    fn guarded_preferences_string_helper_uses_display_context_and_schema() {
        let key = parse_key("editor.test.string").unwrap();
        let mut registry = settings_registry_with_defaults();
        registry
            .register(
                SettingDefinition::new(
                    key.clone(),
                    SettingsScope::Session,
                    SettingSchema::String { maximum_bytes: 12 },
                    SettingValue::String("".to_owned()),
                    false,
                    SettingsPresentation::new(
                        "settings.test.label",
                        "settings.test.description",
                        ["settings.category.editor"],
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        let authority = Arc::new(SettingsAuthority::from_registry_for_performance_benchmark(
            registry,
        ));
        let persistence = SettingsPersistenceService::new(
            Arc::clone(&authority),
            crate::core::jobs::test_job_scheduler(),
        );
        let mutations = SettingsMutationCoordinator::new(Arc::clone(&authority), persistence, None);
        let displayed = mutations.capture_values([&key]).unwrap();
        assert!(matches!(
            set_string_override_from_context(
                &mutations,
                &displayed,
                key.as_str(),
                "over twelve bytes"
            ),
            Err(SettingsUiMutationError::Mutation(
                SettingsMutationError::Settings(SettingsError::InvalidValue { .. })
            ))
        ));
        let receipt =
            set_string_override_from_context(&mutations, &displayed, key.as_str(), "中文").unwrap();
        assert!(receipt.changed());
        assert_eq!(receipt.scope(), SettingsScope::Session);
        assert!(matches!(
            set_string_override_from_context(&mutations, &displayed, key.as_str(), "stale"),
            Err(SettingsUiMutationError::Mutation(
                SettingsMutationError::Settings(SettingsError::StaleGeneration { .. })
            ))
        ));
        assert_eq!(
            authority.resolved_setting(&key).unwrap().value(),
            &SettingValue::String("中文".to_owned())
        );
        let current = mutations.capture_values([&key]).unwrap();
        assert!(
            reset_override_from_context(&mutations, &current, key.as_str())
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn guarded_preferences_reset_keeps_inherited_user_and_session_overrides() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(crate::core::settings::VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        for (scope, value) in [(SettingsScope::User, 2.5), (SettingsScope::Session, 7.5)] {
            mutations
                .authority()
                .set(scope, &key, SettingValue::Float(value))
                .unwrap();
        }
        let displayed = mutations.capture_values([&key]).unwrap();
        let before_health = mutations.persistence_health_snapshot();
        assert_eq!(
            reset_override_from_context(&mutations, &displayed, key.as_str()),
            Ok(None)
        );
        assert_eq!(mutations.persistence_health_snapshot(), before_health);
        let after = mutations.capture_values([&key]).unwrap();
        assert_eq!(after, displayed);
        assert_eq!(
            after.values().values()[0].override_in_scope(SettingsScope::User),
            Some(&SettingValue::Float(2.5))
        );
        assert_eq!(
            after.values().values()[0].override_in_scope(SettingsScope::Session),
            Some(&SettingValue::Float(7.5))
        );
    }

    #[test]
    fn guarded_preferences_existing_control_actions_reject_stale_values_before_applying() {
        let cases = [
            (
                "bool",
                SettingSchema::Bool,
                SettingValue::Bool(false),
                SettingValue::Bool(true),
                CapturedSettingsEdit::ToggleBool,
                SettingValue::Bool(false),
            ),
            (
                "int",
                SettingSchema::Int {
                    minimum: 0,
                    maximum: i64::MAX,
                    step: 2,
                },
                SettingValue::Int(i64::MAX - 5),
                SettingValue::Int(i64::MAX - 3),
                CapturedSettingsEdit::NumericStep(SettingNumericStepDirection::Increment),
                SettingValue::Int(i64::MAX - 1),
            ),
            (
                "float",
                SettingSchema::Float {
                    minimum: 0.0,
                    maximum: 10.0,
                    step: 0.5,
                },
                SettingValue::Float(1.0),
                SettingValue::Float(2.5),
                CapturedSettingsEdit::NumericStep(SettingNumericStepDirection::Increment),
                SettingValue::Float(3.0),
            ),
            (
                "color",
                SettingSchema::Color { channel_step: 16 },
                SettingValue::Color([0, 16, 32, 255]),
                SettingValue::Color([250, 16, 32, 255]),
                CapturedSettingsEdit::ColorStep(
                    SettingColorChannel::Red,
                    SettingNumericStepDirection::Increment,
                ),
                SettingValue::Color([255, 16, 32, 255]),
            ),
            (
                "enum",
                SettingSchema::Enum {
                    variants: ["en".to_owned(), "zh-CN".to_owned()].into_iter().collect(),
                },
                SettingValue::Enum("en".to_owned()),
                SettingValue::Enum("zh-CN".to_owned()),
                CapturedSettingsEdit::Enum("en"),
                SettingValue::Enum("en".to_owned()),
            ),
            (
                "chord",
                SettingSchema::Chord,
                SettingValue::Chord("Ctrl+S".parse().unwrap()),
                SettingValue::Chord("Ctrl+O".parse().unwrap()),
                CapturedSettingsEdit::Chord("Ctrl+P"),
                SettingValue::Chord("Ctrl+P".parse().unwrap()),
            ),
        ];
        for (suffix, schema, default, direct, edit, expected) in cases {
            let key = parse_key(&format!("editor.test.guarded_{suffix}")).unwrap();
            let mut registry = settings_registry_with_defaults();
            registry
                .register(
                    SettingDefinition::new(
                        key.clone(),
                        SettingsScope::Session,
                        schema,
                        default,
                        false,
                        SettingsPresentation::new(
                            "settings.test.label",
                            "settings.test.description",
                            ["settings.category.editor"],
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap();
            let authority = Arc::new(SettingsAuthority::from_registry_for_performance_benchmark(
                registry,
            ));
            let persistence = SettingsPersistenceService::new(
                Arc::clone(&authority),
                crate::core::jobs::test_job_scheduler(),
            );
            let mutations =
                SettingsMutationCoordinator::new(Arc::clone(&authority), persistence, None);
            let displayed = mutations.capture_values([&key]).unwrap();
            authority
                .set(SettingsScope::Session, &key, direct.clone())
                .unwrap();
            let generation = authority.snapshot().generation();
            let health = mutations.persistence_health_snapshot();
            assert!(
                matches!(
                    edit_setting_from_context(&mutations, &displayed, key.as_str(), edit),
                    Err(SettingsUiMutationError::Mutation(
                        SettingsMutationError::Settings(SettingsError::StaleGeneration { .. })
                    ))
                ),
                "{suffix}"
            );
            assert_eq!(authority.snapshot().generation(), generation);
            assert_eq!(authority.resolved_setting(&key).unwrap().value(), &direct);
            assert_eq!(mutations.persistence_health_snapshot(), health);

            let current = mutations.capture_values([&key]).unwrap();
            let receipt =
                edit_setting_from_context(&mutations, &current, key.as_str(), edit).unwrap();
            assert!(receipt.changed(), "{suffix}");
            assert_eq!(receipt.scope(), SettingsScope::Session);
            assert_eq!(receipt.persistence_generation(), None);
            assert_eq!(authority.resolved_setting(&key).unwrap().value(), &expected);
            mutations.flush_then_shutdown().unwrap().finish().unwrap();
        }
    }

    #[test]
    fn guarded_preferences_control_validation_and_missing_capture_do_not_change_authority() {
        use crate::core::settings::{EDITOR_LOCALE_KEY, VIEWPORT_TRANSLATE_STEP_KEY};
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let number = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        let locale = parse_key(EDITOR_LOCALE_KEY).unwrap();
        let displayed = mutations.capture_values([&number, &locale]).unwrap();
        let before = mutations.authority().snapshot().generation();
        assert!(matches!(
            edit_setting_from_context(
                &mutations,
                &displayed,
                number.as_str(),
                CapturedSettingsEdit::ToggleBool
            ),
            Err(SettingsUiMutationError::NotBoolean(_))
        ));
        assert!(matches!(
            edit_setting_from_context(
                &mutations,
                &displayed,
                number.as_str(),
                CapturedSettingsEdit::Chord("Ctrl+S")
            ),
            Err(SettingsUiMutationError::NotChord(_))
        ));
        assert!(matches!(
            edit_setting_from_context(
                &mutations,
                &displayed,
                number.as_str(),
                CapturedSettingsEdit::ColorStep(
                    SettingColorChannel::Red,
                    SettingNumericStepDirection::Increment
                )
            ),
            Err(SettingsUiMutationError::NotColor(_))
        ));
        assert!(matches!(
            edit_setting_from_context(
                &mutations,
                &displayed,
                locale.as_str(),
                CapturedSettingsEdit::Enum("unknown")
            ),
            Err(SettingsUiMutationError::InvalidEnumVariant { .. })
        ));
        let empty = mutations.capture_values(std::iter::empty()).unwrap();
        assert!(matches!(
            edit_setting_from_context(
                &mutations,
                &empty,
                number.as_str(),
                CapturedSettingsEdit::NumericStep(SettingNumericStepDirection::Increment)
            ),
            Err(SettingsUiMutationError::Mutation(
                SettingsMutationError::KeyNotCaptured(_)
            ))
        ));
        assert_eq!(mutations.authority().snapshot().generation(), before);
        assert!(mutations.context_is_current(&displayed));
        mutations.flush_then_shutdown().unwrap().finish().unwrap();
    }
}

#[cfg(test)]
mod settings_scope_mutation_tests {
    use super::*;
    use crate::core::settings::{
        SettingValueSource, EDITOR_SHOW_PREVIEW_FEATURES_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
    };
    use crate::tests::support::OwnedSourceWriteFixture;

    #[test]
    fn numeric_text_commit_preserves_declared_type_and_session_receipt() {
        for (key, text, expected) in [
            (
                VIEWPORT_TRANSLATE_STEP_KEY,
                " 2.5e0 ",
                SettingValue::Float(2.5),
            ),
            (
                crate::core::settings::EDITOR_AUTOSAVE_INTERVAL_SECS_KEY,
                " 45 ",
                SettingValue::Int(45),
            ),
        ] {
            let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
            let receipt =
                set_numeric_override(&mutations, key, SettingsScope::Session, text).unwrap();
            assert_eq!(receipt.scope(), SettingsScope::Session);
            assert_eq!(
                receipt.disposition(),
                crate::core::settings::SettingsMutationDisposition::SessionApplied
            );
            assert_eq!(receipt.persistence_generation(), None);
            let resolved = mutations
                .authority()
                .resolved_setting(&parse_key(key).unwrap())
                .unwrap();
            assert_eq!(resolved.value(), &expected);
            assert_eq!(resolved.source(), SettingValueSource::Session);
            let repeated =
                set_numeric_override(&mutations, key, SettingsScope::Session, text).unwrap();
            assert!(!repeated.changed());
            assert_eq!(
                repeated.authority_generation(),
                receipt.authority_generation()
            );
        }
    }

    #[test]
    fn invalid_numeric_text_never_mutates_or_clamps_the_setting() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let generation = mutations.authority().snapshot().generation();
        for (key, texts) in [
            (
                VIEWPORT_TRANSLATE_STEP_KEY,
                &["", "三", "1,5", "NaN", "inf", "1e309", "0", "10000000"][..],
            ),
            (
                crate::core::settings::EDITOR_AUTOSAVE_INTERVAL_SECS_KEY,
                &["15.5", "3e1", "9223372036854775808", "14", "3601"][..],
            ),
        ] {
            for text in texts {
                assert!(
                    matches!(
                        set_numeric_override(&mutations, key, SettingsScope::Session, text),
                        Err(SettingsUiMutationError::InvalidNumericText { .. })
                            | Err(SettingsUiMutationError::Settings(
                                SettingsError::InvalidValue { .. }
                            ))
                    ),
                    "unexpected result for {key}={text}"
                );
            }
            assert_eq!(
                mutations
                    .authority()
                    .resolved_setting(&parse_key(key).unwrap())
                    .unwrap()
                    .source(),
                SettingValueSource::Default
            );
        }
        assert_eq!(mutations.authority().snapshot().generation(), generation);
    }

    #[test]
    fn numeric_text_commit_rejects_unknown_nonnumeric_and_forbidden_scope() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let generation = mutations.authority().snapshot().generation();
        let unknown = "editor.numeric.unregistered";
        assert_eq!(
            set_numeric_override(&mutations, unknown, SettingsScope::Session, "30"),
            Err(SettingsUiMutationError::UnknownSetting(
                parse_key(unknown).unwrap()
            ))
        );
        assert_eq!(
            set_numeric_override(
                &mutations,
                EDITOR_SHOW_PREVIEW_FEATURES_KEY,
                SettingsScope::Session,
                "30"
            ),
            Err(SettingsUiMutationError::NotNumeric(
                parse_key(EDITOR_SHOW_PREVIEW_FEATURES_KEY).unwrap()
            ))
        );
        let key = crate::core::settings::EDITOR_AUTOSAVE_INTERVAL_SECS_KEY;
        assert_eq!(
            set_numeric_override(&mutations, key, SettingsScope::Project, "30"),
            Err(SettingsUiMutationError::Settings(
                SettingsError::ScopeNotAllowed {
                    key: key.to_owned(),
                    requested: SettingsScope::Project,
                    defined: SettingsScope::User,
                }
            ))
        );
        assert_eq!(mutations.authority().snapshot().generation(), generation);
    }

    #[test]
    fn explicit_session_write_keeps_the_inherited_user_layer() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        mutations
            .authority()
            .set(SettingsScope::User, &key, SettingValue::Float(2.5))
            .unwrap();

        let receipt = set_override(
            &mutations,
            &key,
            SettingsScope::Session,
            SettingValue::Float(5.0),
        )
        .unwrap();

        assert!(receipt.changed());
        assert_eq!(receipt.scope(), SettingsScope::Session);
        assert_eq!(
            receipt.disposition(),
            crate::core::settings::SettingsMutationDisposition::SessionApplied
        );
        assert_eq!(receipt.persistence_generation(), None);
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .value(),
            &SettingValue::Float(5.0)
        );
        reset_override(&mutations, key.as_str(), Some(SettingsScope::Session)).unwrap();
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .value(),
            &SettingValue::Float(2.5)
        );
    }

    #[test]
    fn repeated_scoped_write_does_not_publish_another_generation() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        set_override(
            &mutations,
            &key,
            SettingsScope::Session,
            SettingValue::Float(5.0),
        )
        .unwrap();
        let generation = mutations.authority().snapshot().generation();

        let receipt = set_override(
            &mutations,
            &key,
            SettingsScope::Session,
            SettingValue::Float(5.0),
        )
        .unwrap();

        assert!(!receipt.changed());
        assert_eq!(receipt.authority_generation(), generation);
        assert_eq!(mutations.authority().snapshot().generation(), generation);
    }

    #[test]
    fn project_theme_write_rejects_the_user_only_key_before_mutation() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(crate::core::settings::EDITOR_DESIGN_TOKENS_KEY).unwrap();
        let value = mutations
            .authority()
            .resolved_setting(&key)
            .unwrap()
            .value()
            .clone();
        let generation = mutations.authority().snapshot().generation();

        let error = set_override(&mutations, &key, SettingsScope::Project, value).unwrap_err();

        assert_eq!(
            error,
            SettingsUiMutationError::Settings(SettingsError::ScopeNotAllowed {
                key: key.as_str().to_owned(),
                requested: SettingsScope::Project,
                defined: SettingsScope::User,
            })
        );
        assert_eq!(mutations.authority().snapshot().generation(), generation);
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .source(),
            SettingValueSource::Default
        );
    }

    #[test]
    fn scoped_numeric_write_rejects_wrong_type_nonfinite_and_out_of_range_values() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        let generation = mutations.authority().snapshot().generation();

        for value in [
            SettingValue::Int(5),
            SettingValue::Float(f64::NAN),
            SettingValue::Float(10_000_000.0),
        ] {
            assert!(matches!(
                set_override(&mutations, &key, SettingsScope::Session, value),
                Err(SettingsUiMutationError::Settings(
                    SettingsError::InvalidValue { .. }
                ))
            ));
        }

        assert_eq!(mutations.authority().snapshot().generation(), generation);
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .source(),
            SettingValueSource::Default
        );
    }

    #[test]
    fn unbound_project_write_preserves_the_inherited_user_value() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        mutations
            .authority()
            .set(SettingsScope::User, &key, SettingValue::Float(2.5))
            .unwrap();
        let generation = mutations.authority().snapshot().generation();

        assert_eq!(
            set_override(
                &mutations,
                &key,
                SettingsScope::Project,
                SettingValue::Float(5.0)
            ),
            Err(SettingsUiMutationError::Mutation(
                SettingsMutationError::ProjectNotBound
            ))
        );
        assert_eq!(mutations.authority().snapshot().generation(), generation);
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .value(),
            &SettingValue::Float(2.5)
        );
    }

    #[test]
    fn scoped_write_cannot_register_an_unknown_setting() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key("editor.missing.setting").unwrap();
        let generation = mutations.authority().snapshot().generation();

        assert_eq!(
            set_override(
                &mutations,
                &key,
                SettingsScope::Session,
                SettingValue::Bool(true)
            ),
            Err(SettingsUiMutationError::UnknownSetting(key.clone()))
        );
        assert_eq!(mutations.authority().snapshot().generation(), generation);
        assert!(mutations
            .authority()
            .snapshot()
            .catalog()
            .definition(&key)
            .is_none());
    }

    #[test]
    fn default_value_reset_does_not_require_a_writable_user_source() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let generation = mutations.authority().snapshot().generation();

        assert_eq!(
            reset_override(&mutations, EDITOR_SHOW_PREVIEW_FEATURES_KEY, None),
            Ok(None)
        );
        assert_eq!(mutations.authority().snapshot().generation(), generation);
    }

    #[test]
    fn project_reset_without_override_preserves_inherited_user_value() {
        let project = OwnedSourceWriteFixture::new("settings_project_reset").unwrap();
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        mutations.bind_project(project.root()).unwrap();
        let key = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        mutations
            .authority()
            .set(SettingsScope::User, &key, SettingValue::Float(2.5))
            .unwrap();
        let generation = mutations.authority().snapshot().generation();

        // The existing mixed settings surface uses the catalog's declared Project layer.
        assert_eq!(reset_override(&mutations, key.as_str(), None), Ok(None));

        let resolved = mutations.authority().resolved_setting(&key).unwrap();
        assert_eq!(resolved.value(), &SettingValue::Float(2.5));
        assert_eq!(
            resolved.source(),
            SettingValueSource::Scope(SettingsScope::User)
        );
        assert_eq!(mutations.authority().snapshot().generation(), generation);
        assert!(!project.root().join(".zircon").exists());
    }

    #[test]
    fn project_reset_does_not_follow_a_session_override() {
        let project = OwnedSourceWriteFixture::new("settings_session_reset").unwrap();
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        mutations.bind_project(project.root()).unwrap();
        let key = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        mutations
            .authority()
            .set(SettingsScope::User, &key, SettingValue::Float(2.5))
            .unwrap();
        mutations
            .set(SettingsScope::Session, &key, SettingValue::Float(5.0))
            .unwrap();
        let generation = mutations.authority().snapshot().generation();

        assert_eq!(
            reset_override(&mutations, key.as_str(), Some(SettingsScope::Project)),
            Ok(None)
        );

        let resolved = mutations.authority().resolved_setting(&key).unwrap();
        assert_eq!(resolved.value(), &SettingValue::Float(5.0));
        assert_eq!(
            resolved.source(),
            SettingValueSource::Scope(SettingsScope::Session)
        );
        assert_eq!(mutations.authority().snapshot().generation(), generation);
        mutations.clear(SettingsScope::Session, &key).unwrap();
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .value(),
            &SettingValue::Float(2.5)
        );
        assert!(!project.root().join(".zircon").exists());
    }

    #[test]
    fn absent_project_reset_under_session_needs_no_project_binding() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        mutations
            .authority()
            .set(SettingsScope::User, &key, SettingValue::Float(2.5))
            .unwrap();
        mutations
            .set(SettingsScope::Session, &key, SettingValue::Float(5.0))
            .unwrap();
        let before = mutations
            .authority()
            .resolved_settings(std::slice::from_ref(&key))
            .unwrap();
        let health_generation = mutations.persistence_health_snapshot().generation();
        assert_eq!(
            before.values()[0].override_in_scope(SettingsScope::Project),
            None
        );

        assert_eq!(
            reset_override(&mutations, key.as_str(), Some(SettingsScope::Project)),
            Ok(None)
        );

        let after = mutations
            .authority()
            .resolved_settings(std::slice::from_ref(&key))
            .unwrap();
        assert_eq!(before, after);
        assert_eq!(
            after.values()[0].source(),
            SettingValueSource::Scope(SettingsScope::Session)
        );
        assert_eq!(
            mutations.persistence_health_snapshot().generation(),
            health_generation
        );
    }

    #[test]
    fn absent_user_reset_under_session_needs_no_user_persistence_source() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(EDITOR_SHOW_PREVIEW_FEATURES_KEY).unwrap();
        mutations
            .set(SettingsScope::Session, &key, SettingValue::Bool(true))
            .unwrap();
        let before = mutations
            .authority()
            .resolved_settings(std::slice::from_ref(&key))
            .unwrap();
        let health_generation = mutations.persistence_health_snapshot().generation();
        assert_eq!(
            before.values()[0].override_in_scope(SettingsScope::User),
            None
        );

        assert_eq!(reset_override(&mutations, key.as_str(), None), Ok(None));

        let after = mutations
            .authority()
            .resolved_settings(std::slice::from_ref(&key))
            .unwrap();
        assert_eq!(before, after);
        assert_eq!(after.values()[0].value(), &SettingValue::Bool(true));
        assert_eq!(
            mutations.persistence_health_snapshot().generation(),
            health_generation
        );
    }

    #[test]
    fn explicit_session_reset_returns_the_real_receipt_and_reveals_user_value() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        mutations
            .authority()
            .set(SettingsScope::User, &key, SettingValue::Float(2.5))
            .unwrap();
        mutations
            .set(SettingsScope::Session, &key, SettingValue::Float(5.0))
            .unwrap();

        let receipt = reset_override(&mutations, key.as_str(), Some(SettingsScope::Session))
            .unwrap()
            .unwrap();

        assert!(receipt.changed());
        assert_eq!(receipt.scope(), SettingsScope::Session);
        assert_eq!(
            receipt.authority_generation(),
            mutations.authority().snapshot().generation()
        );
        let resolved = mutations.authority().resolved_setting(&key).unwrap();
        assert_eq!(resolved.value(), &SettingValue::Float(2.5));
        assert_eq!(
            resolved.source(),
            SettingValueSource::Scope(SettingsScope::User)
        );
    }

    #[test]
    fn project_reset_rejects_a_user_only_key_before_mutation() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let generation = mutations.authority().snapshot().generation();

        let error = reset_override(
            &mutations,
            EDITOR_SHOW_PREVIEW_FEATURES_KEY,
            Some(SettingsScope::Project),
        )
        .unwrap_err();

        assert_eq!(
            error,
            SettingsUiMutationError::Settings(SettingsError::ScopeNotAllowed {
                key: EDITOR_SHOW_PREVIEW_FEATURES_KEY.to_owned(),
                requested: SettingsScope::Project,
                defined: SettingsScope::User,
            })
        );
        assert_eq!(mutations.authority().snapshot().generation(), generation);
    }

    #[test]
    fn unbound_project_reset_rejects_without_deleting_any_override() {
        let mutations = SettingsMutationCoordinator::in_memory_with_defaults();
        let key = parse_key(VIEWPORT_TRANSLATE_STEP_KEY).unwrap();
        mutations
            .authority()
            .set(SettingsScope::User, &key, SettingValue::Float(2.5))
            .unwrap();
        mutations
            .authority()
            .set(SettingsScope::Project, &key, SettingValue::Float(5.0))
            .unwrap();
        mutations
            .authority()
            .set(SettingsScope::Session, &key, SettingValue::Float(7.5))
            .unwrap();
        let generation = mutations.authority().snapshot().generation();

        assert_eq!(
            reset_override(&mutations, key.as_str(), Some(SettingsScope::Project)),
            Err(SettingsUiMutationError::Mutation(
                SettingsMutationError::ProjectNotBound
            ))
        );
        assert_eq!(mutations.authority().snapshot().generation(), generation);
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .source(),
            SettingValueSource::Scope(SettingsScope::Session)
        );
        mutations
            .authority()
            .clear(SettingsScope::Session, &key)
            .unwrap();
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .value(),
            &SettingValue::Float(5.0)
        );
        mutations
            .authority()
            .clear(SettingsScope::Project, &key)
            .unwrap();
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .value(),
            &SettingValue::Float(2.5)
        );
    }
}

#[cfg(test)]
mod guarded_numeric_text_tests {
    use super::*;
    use crate::core::settings::{
        SettingsAuthority, SettingsPersistenceLimits, SettingsPersistenceService, SettingsStore,
        EDITOR_AUTOSAVE_INTERVAL_SECS_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
    };
    use std::sync::Arc;

    fn pending_coordinator(root: &std::path::Path) -> SettingsMutationCoordinator {
        let authority = Arc::new(SettingsAuthority::with_defaults());
        let persistence = SettingsPersistenceService::with_limits(
            Arc::clone(&authority),
            crate::core::jobs::test_job_scheduler(),
            SettingsPersistenceLimits {
                max_entries: 1,
                max_retained_bytes: 1,
            },
        );
        SettingsMutationCoordinator::new(
            authority,
            persistence,
            Some(SettingsStore::from_roots(root, None)),
        )
    }

    #[test]
    fn captured_numeric_text_uses_builtin_types_scope_and_original_pending_receipt() {
        let fixture =
            crate::tests::support::OwnedSourceWriteFixture::new("captured_numeric_receipt")
                .unwrap();
        let mutations = pending_coordinator(fixture.root());
        mutations.bind_project(fixture.root()).unwrap();
        for (key, text, expected) in [
            (
                EDITOR_AUTOSAVE_INTERVAL_SECS_KEY,
                " 45 ",
                SettingValue::Int(45),
            ),
            (
                VIEWPORT_TRANSLATE_STEP_KEY,
                " 2.5e0 ",
                SettingValue::Float(2.5),
            ),
        ] {
            let key = parse_key(key).unwrap();
            let scope = mutations
                .authority()
                .snapshot()
                .catalog()
                .definition(&key)
                .unwrap()
                .scope;
            let displayed = mutations.capture_values([&key]).unwrap();
            let receipt = edit_setting_from_context(
                &mutations,
                &displayed,
                key.as_str(),
                CapturedSettingsEdit::NumericText(text),
            )
            .unwrap();
            assert_eq!(receipt.scope(), scope);
            assert!(matches!(
                receipt.disposition(),
                crate::core::settings::SettingsMutationDisposition::AppliedPendingAdmission(_)
            ));
            assert_eq!(
                mutations
                    .authority()
                    .resolved_setting(&key)
                    .unwrap()
                    .value(),
                &expected
            );
            let current = mutations.capture_values([&key]).unwrap();
            let health = mutations.persistence_health_snapshot();
            let noop = edit_setting_from_context(
                &mutations,
                &current,
                key.as_str(),
                CapturedSettingsEdit::NumericText(text),
            )
            .unwrap();
            assert!(!noop.changed());
            assert_eq!(noop.persistence_generation(), None);
            assert_eq!(mutations.persistence_health_snapshot(), health);
            let retry = mutations.retry_pending(scope).unwrap();
            assert_eq!(retry.file_generation(), receipt.persistence_generation());
        }
        mutations.shutdown().wait();
    }

    #[test]
    fn captured_numeric_text_rejects_invalid_and_stale_input_without_replacing_pending() {
        let fixture =
            crate::tests::support::OwnedSourceWriteFixture::new("captured_numeric_stale").unwrap();
        let mutations = pending_coordinator(fixture.root());
        let key = parse_key(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY).unwrap();
        mutations
            .set(SettingsScope::User, &key, SettingValue::Int(45))
            .unwrap();
        let displayed = mutations.capture_values([&key]).unwrap();
        let health = mutations.persistence_health_snapshot();
        let generation = mutations.authority().snapshot().generation();
        for text in ["", "三", "15.5", "3e1", "9223372036854775808", "14", "3601"] {
            assert!(edit_setting_from_context(
                &mutations,
                &displayed,
                key.as_str(),
                CapturedSettingsEdit::NumericText(text)
            )
            .is_err());
            assert_eq!(mutations.authority().snapshot().generation(), generation);
            assert_eq!(mutations.persistence_health_snapshot(), health);
        }
        mutations
            .authority()
            .set(SettingsScope::Session, &key, SettingValue::Int(60))
            .unwrap();
        assert!(matches!(
            edit_setting_from_context(
                &mutations,
                &displayed,
                key.as_str(),
                CapturedSettingsEdit::NumericText("60")
            ),
            Err(SettingsUiMutationError::Mutation(
                SettingsMutationError::Settings(SettingsError::StaleGeneration { .. })
            ))
        ));
        assert_eq!(mutations.persistence_health_snapshot(), health);
        assert_eq!(
            mutations
                .authority()
                .resolved_setting(&key)
                .unwrap()
                .value(),
            &SettingValue::Int(60)
        );
        mutations.shutdown().wait();
    }

    #[test]
    fn numeric_text_parser_keeps_full_i64_and_rejects_fractional_integer_syntax() {
        let key = parse_key(EDITOR_AUTOSAVE_INTERVAL_SECS_KEY).unwrap();
        // Parser boundary coverage only; this does not widen the built-in autosave schema.
        let schema = SettingSchema::Int {
            minimum: i64::MIN,
            maximum: i64::MAX,
            step: 1,
        };
        for value in [i64::MIN, 9_007_199_254_740_993, i64::MAX] {
            assert_eq!(
                parse_numeric_setting_value(&schema, &key, &value.to_string()).unwrap(),
                SettingValue::Int(value)
            );
        }
        for value in ["9223372036854775808", "1.0", "1e1"] {
            assert!(parse_numeric_setting_value(&schema, &key, value).is_err());
        }
    }
}
