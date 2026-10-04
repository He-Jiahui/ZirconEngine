use crate::core::script_build::{
    ScriptBuildOrchestrator, ScriptBuildOutcome, DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS,
    DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS, MAX_INCREMENTAL_SCRIPT_WATCH_PATHS,
};
use crate::core::settings::{
    settings_registry_with_defaults, SettingSchema, SettingValue, SettingsAuthority, SettingsError,
    SettingsKey, SettingsScope,
};

use super::{
    MAXIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS, MINIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS,
    SCRIPT_BUILD_BATCH_WINDOW_MS_KEY, SCRIPT_BUILD_BATCH_WINDOW_STEP_MS,
};

fn batch_window_key() -> SettingsKey {
    SettingsKey::parse(SCRIPT_BUILD_BATCH_WINDOW_MS_KEY)
        .expect("the built-in script-build batch-window key is valid")
}

#[test]
fn batch_window_is_a_bounded_hot_applied_user_setting() {
    let registry = settings_registry_with_defaults();
    let definition = registry
        .definition(&batch_window_key())
        .expect("the script-build batch window is registered by the default editor registry");

    assert_eq!(definition.scope, SettingsScope::User);
    assert_eq!(
        definition.schema,
        SettingSchema::Int {
            minimum: MINIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS,
            maximum: MAXIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS,
            step: SCRIPT_BUILD_BATCH_WINDOW_STEP_MS,
        }
    );
    assert_eq!(
        definition.default,
        SettingValue::Int(
            i64::try_from(DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS)
                .expect("the default debounce fits the settings integer schema"),
        )
    );
    assert!(!definition.requires_restart);
    assert_eq!(
        definition.presentation().label_key(),
        "settings.editor.script_build.batch_window_ms.label"
    );
    assert_eq!(
        definition.presentation().description_key(),
        "settings.editor.script_build.batch_window_ms.description"
    );
}

#[test]
fn batch_window_rejects_values_outside_the_registered_range() {
    let authority = SettingsAuthority::with_defaults();
    let key = batch_window_key();

    for value in [
        MINIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS - 1,
        MAXIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS + 1,
    ] {
        assert!(matches!(
            authority.set(SettingsScope::User, &key, SettingValue::Int(value)),
            Err(SettingsError::InvalidValue { .. })
        ));
    }
    assert!(matches!(
        authority.set(SettingsScope::User, &key, SettingValue::Bool(true)),
        Err(SettingsError::InvalidValue { .. })
    ));
}

#[test]
fn batch_window_presentation_has_direct_embedded_translations() {
    for (locale, bundle) in [
        ("en", include_str!("../../../../../assets/i18n/en.toml")),
        (
            "zh-CN",
            include_str!("../../../../../assets/i18n/zh-CN.toml"),
        ),
    ] {
        let document = toml::from_str::<toml::Table>(bundle)
            .unwrap_or_else(|error| panic!("{locale} embedded bundle must parse: {error}"));
        assert_eq!(
            document.get("locale").and_then(toml::Value::as_str),
            Some(locale)
        );
        let translations = document
            .get("translations")
            .and_then(toml::Value::as_table)
            .unwrap_or_else(|| panic!("{locale} embedded bundle must contain translations"));
        for key in [
            "settings.category.script_build",
            "settings.editor.script_build.batch_window_ms.label",
            "settings.editor.script_build.batch_window_ms.description",
        ] {
            assert!(
                translations.contains_key(key),
                "{key} must have a direct translation in {locale}"
            );
        }
    }
}

#[test]
fn current_shell_value_roundtrips_into_new_and_existing_orchestrators() {
    let authority = SettingsAuthority::with_defaults();
    let key = batch_window_key();
    let mut existing = ScriptBuildOrchestrator::from_settings(&authority).unwrap();

    assert_eq!(
        existing.batch_policy().debounce_ms(),
        DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS
    );
    authority
        .set(SettingsScope::User, &key, SettingValue::Int(600))
        .unwrap()
        .expect("a changed User value advances the current settings generation");

    assert_eq!(
        authority.resolved_setting(&key).unwrap().value(),
        &SettingValue::Int(600)
    );
    assert!(existing.synchronize_settings(&authority).unwrap());
    assert_eq!(existing.batch_policy().debounce_ms(), 600);
    assert_eq!(
        ScriptBuildOrchestrator::from_settings(&authority)
            .unwrap()
            .batch_policy()
            .debounce_ms(),
        600
    );
    assert!(!existing.synchronize_settings(&authority).unwrap());
}

#[test]
fn hot_change_recomputes_pending_deadline_without_moving_first_event_hard_limit() {
    let authority = SettingsAuthority::with_defaults();
    let key = batch_window_key();
    let mut orchestrator = ScriptBuildOrchestrator::from_settings(&authority).unwrap();
    orchestrator.notify_watch_change("Scripts/First.zr", 100);
    orchestrator.notify_watch_change("Scripts/Last.zr", 200);
    assert_eq!(orchestrator.snapshot().watch_deadline_ms(), Some(500));

    authority
        .set(
            SettingsScope::User,
            &key,
            SettingValue::Int(MAXIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS),
        )
        .unwrap();
    assert!(orchestrator.synchronize_settings(&authority).unwrap());
    assert_eq!(
        orchestrator.snapshot().watch_deadline_ms(),
        Some(100 + DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS),
        "a larger batch window remains capped by the first observed event"
    );

    authority
        .set(
            SettingsScope::User,
            &key,
            SettingValue::Int(MINIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS),
        )
        .unwrap();
    assert!(orchestrator.synchronize_settings(&authority).unwrap());
    assert_eq!(
        orchestrator.snapshot().watch_deadline_ms(),
        Some(200 + u64::try_from(MINIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS).unwrap()),
        "a smaller batch window is applied to the last observed event"
    );
    assert_eq!(
        orchestrator.snapshot().watch_first_observed_at_ms(),
        Some(100)
    );
}

#[test]
fn hot_change_preserves_watch_budget_single_flight_and_cancellation_identity() {
    let authority = SettingsAuthority::with_defaults();
    let key = batch_window_key();
    let mut orchestrator = ScriptBuildOrchestrator::from_settings(&authority).unwrap();
    let active_id = orchestrator.enqueue_command().unwrap();
    let active_dispatch = orchestrator.take_ready(0).unwrap().unwrap();

    for index in 0..=MAX_INCREMENTAL_SCRIPT_WATCH_PATHS {
        orchestrator.notify_watch_change(format!("Scripts/{index}.zr"), 10 + index as u64);
    }
    let before = orchestrator.snapshot();
    assert_eq!(
        before.pending_watch_path_count(),
        MAX_INCREMENTAL_SCRIPT_WATCH_PATHS + 1,
        "the path budget folds overflow into the full-rebuild sentinel"
    );

    authority
        .set(SettingsScope::User, &key, SettingValue::Int(500))
        .unwrap();
    assert!(orchestrator.synchronize_settings(&authority).unwrap());
    let after = orchestrator.snapshot();
    assert_eq!(after.active_request_id(), Some(active_id));
    assert_eq!(after.queued_request_count(), before.queued_request_count());
    assert_eq!(
        after.pending_watch_path_count(),
        before.pending_watch_path_count()
    );

    let queued_id = orchestrator.enqueue_play().unwrap();
    authority
        .set(SettingsScope::User, &key, SettingValue::Int(700))
        .unwrap();
    assert!(orchestrator.synchronize_settings(&authority).unwrap());
    assert_eq!(orchestrator.snapshot().active_request_id(), Some(active_id));
    assert_eq!(orchestrator.snapshot().queued_request_count(), 1);

    let cancelled = orchestrator
        .complete(
            active_dispatch,
            ScriptBuildOutcome::Cancelled {
                reason: "cancel exact active generation".into(),
            },
        )
        .unwrap();
    assert_eq!(cancelled.request_id(), active_id);
    assert_eq!(cancelled.dropped_queued_request_count(), 1);
    let next_id = orchestrator.enqueue_command().unwrap();
    assert_eq!(next_id.get(), queued_id.get() + 1);
}
