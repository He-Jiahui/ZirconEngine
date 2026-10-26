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
