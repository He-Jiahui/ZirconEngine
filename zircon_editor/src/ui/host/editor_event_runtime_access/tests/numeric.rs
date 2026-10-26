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
