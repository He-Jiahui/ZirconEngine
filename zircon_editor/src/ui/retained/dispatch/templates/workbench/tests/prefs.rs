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
