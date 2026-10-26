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
        assert_eq!(
            resolved.source(),
            SettingValueSource::Scope(SettingsScope::Session)
        );
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
