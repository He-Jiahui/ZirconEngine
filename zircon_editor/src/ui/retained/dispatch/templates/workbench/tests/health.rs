use super::*;
use crate::core::i18n::EditorI18nService;
use crate::core::settings::{
    SettingsAuthority, SettingsMutationCoordinator, SettingsMutationDisposition,
    SettingsPersistenceLimits, SettingsPersistenceService, SettingsStore, EDITOR_LOCALE_KEY,
    EDITOR_SHOW_PREVIEW_FEATURES_KEY,
};
use crate::tests::support::OwnedSourceWriteFixture;
use crate::ui::workbench::reference::EditorWorkbenchTemplateSurfaceError;
use std::sync::Arc;
use zircon_runtime_interface::ui::{layout::UiSize, tree::UiTreeError};

fn pending_user_settings() -> (OwnedSourceWriteFixture, SettingsMutationCoordinator) {
    let fixture = OwnedSourceWriteFixture::new("preferences_health_materialization").unwrap();
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
    let receipt = mutations
        .set(SettingsScope::User, &key, SettingValue::Bool(true))
        .unwrap();
    assert!(matches!(
        receipt.disposition(),
        SettingsMutationDisposition::AppliedPendingAdmission(_)
    ));
    (fixture, mutations)
}

fn pending_health(
    mutations: &SettingsMutationCoordinator,
) -> (
    SettingsPersistenceHealthProjection,
    SettingsPersistenceRetryContext,
) {
    let (snapshot, contexts) = mutations.capture_persistence_health_with_retry().unwrap();
    let retry = contexts
        .into_iter()
        .find(|context| context.scope() == SettingsScope::User)
        .unwrap();
    (
        SettingsPersistenceHealthProjection::capture(snapshot, &EditorI18nService::default()),
        retry,
    )
}

#[test]
fn health_materialization_success_and_noop_keep_the_identical_displayed_batch() {
    let (_fixture, mutations) = pending_user_settings();
    let (health, retry) = pending_health(&mutations);
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .open_settings_window(
            guarded_preferences_tests::open_state(
                &mutations,
                "builtin|settings.category.editor",
            )
            .with_persistence_retry_context(Some(retry.clone())),
        )
        .unwrap();
    let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
    assert!(!bridge
        .refresh_settings_persistence_health(&health, Some(&retry))
        .unwrap());
    assert_eq!(bridge.settings_values_context(), Some(&displayed));
    assert_eq!(
        bridge.settings_persistence_retry_context("user").unwrap(),
        retry
    );

    mutations.retry_pending(SettingsScope::User).unwrap();
    let (health, retry) = pending_health(&mutations);
    assert!(bridge
        .refresh_settings_persistence_health(&health, Some(&retry))
        .unwrap());
    assert_eq!(bridge.settings_values_context(), Some(&displayed));
    assert_eq!(
        bridge.settings_persistence_retry_context("user").unwrap(),
        retry
    );
    assert!(!bridge
        .refresh_settings_persistence_health(&health, Some(&retry))
        .unwrap());
    assert_eq!(bridge.settings_values_context(), Some(&displayed));
    assert_eq!(
        mutations.authority().snapshot().generation(),
        displayed.generation()
    );
    mutations.shutdown().wait();
}

#[test]
fn health_materialization_failure_revokes_edits_until_normal_values_refresh() {
    let (_fixture, mutations) = pending_user_settings();
    let (_, retry) = pending_health(&mutations);
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .open_settings_window(
            guarded_preferences_tests::open_state(
                &mutations,
                "builtin|settings.category.editor",
            )
            .with_persistence_retry_context(Some(retry)),
        )
        .unwrap();
    let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
    mutations.retry_pending(SettingsScope::User).unwrap();
    let (health, retry) = pending_health(&mutations);

    // Exercise the actual template refresh error after health property preparation.
    let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
    let removed = bridge
        .template_surface
        .surface
        .tree
        .nodes
        .remove(&missing)
        .unwrap();
    for root in bridge.template_surface.surface.tree.roots.clone() {
        let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
        node.dirty.layout = true;
        node.dirty.input = true;
    }
    let error = bridge
        .refresh_settings_persistence_health(&health, Some(&retry))
        .unwrap_err();
    assert!(
        matches!(
            &error,
            BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
            ) if *id == missing
        ),
        "{error}"
    );
    assert!(bridge.settings_values_context().is_none());
    assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
    assert!(bridge.settings_persistence_retry_context("user").is_err());
    assert!(bridge.settings_persistence_health_needs_refresh);
    assert_eq!(
        mutations.authority().snapshot().generation(),
        displayed.generation()
    );
    assert_eq!(pending_health(&mutations).1, retry);

    bridge
        .template_surface
        .surface
        .tree
        .nodes
        .insert(missing, removed);
    for root in bridge.template_surface.surface.tree.roots.clone() {
        let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
        node.dirty.layout = true;
        node.dirty.input = true;
    }
    assert!(bridge
        .refresh_settings_persistence_health(&health, Some(&retry))
        .unwrap());
    assert_eq!(
        bridge.settings_persistence_retry_context("user").unwrap(),
        retry
    );
    // Successful retry presentation cannot manufacture replacement value authority.
    assert!(bridge.settings_values_context().is_none());
    bridge.refresh_settings_values(&displayed).unwrap();
    assert_eq!(bridge.settings_values_context(), Some(&displayed));
    assert_eq!(pending_health(&mutations).1, retry);
    assert_eq!(
        mutations.authority().snapshot().generation(),
        displayed.generation()
    );
    mutations.shutdown().wait();
}

mod capture_failure {
    use super::*;
    use crate::core::settings::{
        SettingChange, SettingsChangeSubscriber, SettingsMutationError, SettingsSnapshot,
        VIEWPORT_TRANSLATE_STEP_KEY,
    };
    use std::sync::{mpsc, Weak};
    use std::time::Duration;

    #[test]
    fn failed_health_capture_retires_stale_batch_without_consuming_pending_write() {
        struct PhaseProbe {
            mutations: Weak<SettingsMutationCoordinator>,
            displayed: SettingsMutationContext,
            observed: mpsc::Sender<(bool, bool)>,
        }
        impl SettingsChangeSubscriber for PhaseProbe {
            fn settings_changed(&self, _: &[SettingChange], _: &SettingsSnapshot) {
                let mutations = self.mutations.upgrade().unwrap();
                let health_rejected = matches!(
                    mutations.capture_persistence_health_with_retry(),
                    Err(SettingsMutationError::OperationInProgress { .. })
                );
                self.observed
                    .send((
                        health_rejected,
                        mutations.context_is_current(&self.displayed),
                    ))
                    .unwrap();
            }
        }

        let (_fixture, mutations) = pending_user_settings();
        let mutations = Arc::new(mutations);
        mutations
            .authority()
            .set(
                SettingsScope::Project,
                &SettingsKey::parse(VIEWPORT_TRANSLATE_STEP_KEY).unwrap(),
                SettingValue::Float(2.5),
            )
            .unwrap();
        let (health, retry) = pending_health(&mutations);
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                .unwrap();
        bridge
            .open_settings_window(
                guarded_preferences_tests::open_state(
                    &mutations,
                    "builtin|settings.category.editor",
                )
                .with_persistence_retry_context(Some(retry.clone())),
            )
            .unwrap();
        let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
        let (sender, receiver) = mpsc::channel();
        mutations
            .authority()
            .configure_change_subscriber(Arc::new(PhaseProbe {
                mutations: Arc::downgrade(&mutations),
                displayed: displayed.clone(),
                observed: sender,
            }));

        // Observe the actual coordinator phase while its project clear publishes a change.
        mutations.clear_project().unwrap();
        let (health_rejected, values_are_current) =
            receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(health_rejected);
        assert!(!values_are_current);
        assert!(bridge.settings_persistence_capture_failed(values_are_current));
        assert!(bridge.settings_values_context().is_none());
        assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
        assert!(bridge.settings_persistence_retry_context("user").is_err());
        assert_eq!(pending_health(&mutations).1, retry);

        // Recovered health may restore retry, but it cannot reinstall the old value batch.
        bridge
            .refresh_settings_persistence_health(&health, Some(&retry))
            .unwrap();
        assert!(bridge.settings_values_context().is_none());
        assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
        let fresh = guarded_preferences_tests::open_state(
            &mutations,
            "builtin|settings.category.editor",
        )
        .values_context
        .mutation;
        assert!(fresh.generation() > displayed.generation());
        bridge.refresh_settings_values(&fresh).unwrap();
        assert_eq!(bridge.settings_values_context(), Some(&fresh));
        assert_eq!(
            bridge.settings_persistence_retry_context("user").unwrap(),
            retry
        );
        assert_eq!(pending_health(&mutations).1, retry);
        mutations.shutdown().wait();
    }

    #[test]
    fn failed_health_capture_keeps_current_values_and_requires_retry_recapture() {
        let (_fixture, mutations) = pending_user_settings();
        let (health, retry) = pending_health(&mutations);
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                .unwrap();
        bridge
            .open_settings_window(
                guarded_preferences_tests::open_state(
                    &mutations,
                    "builtin|settings.category.editor",
                )
                .with_persistence_retry_context(Some(retry.clone())),
            )
            .unwrap();
        let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
        assert!(mutations.context_is_current(&displayed));
        assert!(!bridge.settings_persistence_capture_failed(true));
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert!(bridge.settings_persistence_retry_context("user").is_err());
        bridge
            .refresh_settings_persistence_health(&health, Some(&retry))
            .unwrap();
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert_eq!(
            bridge.settings_persistence_retry_context("user").unwrap(),
            retry
        );
        assert_eq!(pending_health(&mutations).1, retry);
        assert_eq!(
            mutations.authority().snapshot().generation(),
            displayed.generation()
        );
        mutations.shutdown().wait();
    }
}

mod value_category_state_change {
    use super::*;

    const EDITOR_CATEGORY: &str = "builtin|settings.category.editor";
    const LANGUAGE_CATEGORY: &str =
        "builtin|settings.category.editor/settings.category.language";

    fn category(name: &str) -> &'static str {
        if name == "category" {
            LANGUAGE_CATEGORY
        } else {
            EDITOR_CATEGORY
        }
    }

    fn transition(
        bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
        name: &str,
        values: &SettingsMutationContext,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        match name {
            "values" => bridge.refresh_settings_values(values),
            "values_close" => bridge.refresh_settings_values_and_close_editor(values),
            "category" => bridge
                .select_settings_category(
                    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                    LANGUAGE_CATEGORY,
                    values,
                )
                .map(|handled| handled.expect("settings category must be handled")),
            _ => unreachable!("unknown test transition"),
        }
    }

    fn mark_layout_dirty(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge) {
        for root in bridge.template_surface.surface.tree.roots.clone() {
            let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
            node.dirty.layout = true;
            node.dirty.input = true;
        }
    }

    #[test]
    fn value_and_category_success_preserves_only_supplied_values_and_original_retry() {
        for name in ["values", "values_close", "category"] {
            for with_retry in [true, false] {
                let (_fixture, mutations) = pending_user_settings();
                let (_, retry) = pending_health(&mutations);
                let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
                    1280.0, 800.0,
                ))
                .unwrap();
                bridge
                    .open_settings_window(
                        guarded_preferences_tests::open_state(&mutations, EDITOR_CATEGORY)
                            .with_persistence_retry_context(Some(retry.clone())),
                    )
                    .unwrap();
                assert!(bridge
                    .toggle_settings_editor(
                        EDITOR_LOCALE_KEY,
                        WorkbenchSettingsEditorKind::Enum
                    )
                    .unwrap());
                let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
                let key = SettingsKey::parse(EDITOR_SHOW_PREVIEW_FEATURES_KEY).unwrap();
                assert!(mutations
                    .set(SettingsScope::Session, &key, SettingValue::Bool(false))
                    .unwrap()
                    .changed());
                let supplied =
                    guarded_preferences_tests::open_state(&mutations, category(name))
                        .values_context
                        .mutation;
                assert!(mutations
                    .set(SettingsScope::Session, &key, SettingValue::Bool(true))
                    .unwrap()
                    .changed());
                let current_generation = mutations.authority().snapshot().generation();
                assert_ne!(supplied.generation(), displayed.generation());
                assert_ne!(supplied.generation(), current_generation);
                assert!(!mutations.context_is_current(&supplied));
                if !with_retry {
                    // Receipt callers already revoke retry before preparing health.
                    bridge.invalidate_settings_persistence_retry_context();
                }
                let original_retry = bridge.settings_persistence_retry_context.clone();
                assert_eq!(
                    bridge
                        .select_settings_category("other", LANGUAGE_CATEGORY, &supplied)
                        .unwrap(),
                    None
                );
                for invalid in ["", "builtin|missing"] {
                    assert_eq!(
                        bridge
                            .select_settings_category(
                                WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
                                invalid,
                                &supplied,
                            )
                            .unwrap(),
                        Some(false)
                    );
                }
                assert_eq!(bridge.settings_values_context(), Some(&displayed));
                assert_eq!(bridge.settings_persistence_retry_context, original_retry);

                assert!(transition(&mut bridge, name, &supplied).unwrap());
                assert_eq!(bridge.settings_values_context(), Some(&supplied));
                assert_eq!(bridge.settings_persistence_retry_context, original_retry);
                if with_retry {
                    assert_eq!(
                        bridge.settings_persistence_retry_context("user").unwrap(),
                        retry
                    );
                } else {
                    assert!(bridge.settings_persistence_retry_context("user").is_err());
                }
                if name == "values" {
                    assert_eq!(
                        bridge.settings_editor_open_key(WorkbenchSettingsEditorKind::Enum),
                        Some(EDITOR_LOCALE_KEY.to_owned())
                    );
                } else {
                    assert!(bridge
                        .settings_editor_open_key(WorkbenchSettingsEditorKind::Enum)
                        .is_none());
                }
                assert_eq!(pending_health(&mutations).1, retry);
                assert_eq!(
                    mutations.authority().snapshot().generation(),
                    current_generation
                );
                bridge.close_settings_window().unwrap();
                assert!(!transition(&mut bridge, name, &supplied).unwrap());
                assert!(bridge.settings_values_context().is_none());
                assert!(bridge.settings_persistence_retry_context.is_none());
                mutations.shutdown().wait();
            }
        }
    }

    #[test]
    fn value_and_category_rebuild_failure_revokes_both_contexts_until_recaptured() {
        for name in ["values", "values_close", "category"] {
            let (_fixture, mutations) = pending_user_settings();
            let (health, retry) = pending_health(&mutations);
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                    .unwrap();
            bridge
                .open_settings_window(
                    guarded_preferences_tests::open_state(&mutations, EDITOR_CATEGORY)
                        .with_persistence_retry_context(Some(retry.clone())),
                )
                .unwrap();
            assert!(bridge
                .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
                .unwrap());
            let supplied = guarded_preferences_tests::open_state(&mutations, category(name))
                .values_context
                .mutation;
            let original_generation = mutations.authority().snapshot().generation();
            let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
            let removed = bridge
                .template_surface
                .surface
                .tree
                .nodes
                .remove(&missing)
                .unwrap();
            mark_layout_dirty(&mut bridge);
            let error = transition(&mut bridge, name, &supplied).unwrap_err();
            assert!(
                matches!(
                    &error,
                    BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                        EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
                    ) if *id == missing
                ),
                "{name}: {error}"
            );
            assert!(bridge.settings_values_context().is_none());
            assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
            assert!(bridge.settings_persistence_retry_context.is_none());
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            assert_eq!(pending_health(&mutations).1, retry);
            assert_eq!(
                mutations.authority().snapshot().generation(),
                original_generation
            );

            bridge
                .template_surface
                .surface
                .tree
                .nodes
                .insert(missing, removed);
            mark_layout_dirty(&mut bridge);
            bridge.refresh_prepared_state_change().unwrap();
            assert!(bridge.settings_values_context().is_none());
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            assert!(bridge
                .refresh_settings_persistence_health(&health, Some(&retry))
                .unwrap());
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert!(bridge.settings_values_context().is_none());
            bridge.refresh_settings_values(&supplied).unwrap();
            assert_eq!(bridge.settings_values_context(), Some(&supplied));
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert_eq!(pending_health(&mutations).1, retry);
            assert_eq!(
                mutations.authority().snapshot().generation(),
                original_generation
            );
            mutations.shutdown().wait();
        }
    }
}

mod popup_scroll_state_change {
    use super::*;

    fn prepare_transition(
        bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
        transition: &str,
    ) {
        if matches!(transition, "toggle_close" | "close") {
            assert!(bridge
                .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum)
                .unwrap());
        }
    }

    fn transition(
        bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
        name: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        match name {
            "toggle_open" | "toggle_close" => bridge
                .toggle_settings_editor(EDITOR_LOCALE_KEY, WorkbenchSettingsEditorKind::Enum),
            "close" => bridge.close_settings_editor(),
            "scroll" => bridge.update_settings_scroll_offsets(12.0, 24.0),
            _ => unreachable!("unknown test transition"),
        }
    }

    #[test]
    fn successful_popup_and_scroll_rebuilds_keep_only_the_original_contexts() {
        for name in ["toggle_open", "toggle_close", "close", "scroll"] {
            let (_fixture, mutations) = pending_user_settings();
            let (_, retry) = pending_health(&mutations);
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                    .unwrap();
            bridge
                .open_settings_window(
                    guarded_preferences_tests::open_state(
                        &mutations,
                        "builtin|settings.category.editor",
                    )
                    .with_persistence_retry_context(Some(retry.clone())),
                )
                .unwrap();
            prepare_transition(&mut bridge, name);
            let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
            assert!(!bridge.update_settings_scroll_offsets(0.0, 0.0).unwrap());
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert!(transition(&mut bridge, name).unwrap());
            if name == "scroll" {
                assert!(!bridge.update_settings_scroll_offsets(12.0, 24.0).unwrap());
            } else if name == "close" {
                assert!(!bridge.close_settings_editor().unwrap());
            }
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert_eq!(pending_health(&mutations).1, retry);
            assert_eq!(
                mutations.authority().snapshot().generation(),
                displayed.generation()
            );
            mutations.shutdown().wait();
        }
    }

    #[test]
    fn failed_popup_and_scroll_rebuilds_revoke_both_contexts_until_recaptured() {
        for name in ["toggle_open", "toggle_close", "close", "scroll"] {
            let (_fixture, mutations) = pending_user_settings();
            let (health, retry) = pending_health(&mutations);
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                    .unwrap();
            bridge
                .open_settings_window(
                    guarded_preferences_tests::open_state(
                        &mutations,
                        "builtin|settings.category.editor",
                    )
                    .with_persistence_retry_context(Some(retry.clone())),
                )
                .unwrap();
            prepare_transition(&mut bridge, name);
            let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert!(!bridge.settings_persistence_health_needs_refresh);
            let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
            let removed = bridge
                .template_surface
                .surface
                .tree
                .nodes
                .remove(&missing)
                .unwrap();
            for root in bridge.template_surface.surface.tree.roots.clone() {
                let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                node.dirty.layout = true;
                node.dirty.input = true;
            }
            let error = transition(&mut bridge, name).unwrap_err();
            assert!(
                matches!(
                    &error,
                    BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                        EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
                    ) if *id == missing
                ),
                "{name}: {error}"
            );
            assert!(bridge.settings_values_context().is_none());
            assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
            assert!(bridge.settings_persistence_retry_context.is_none());
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            assert_eq!(pending_health(&mutations).1, retry);
            assert_eq!(
                mutations.authority().snapshot().generation(),
                displayed.generation()
            );

            bridge
                .template_surface
                .surface
                .tree
                .nodes
                .insert(missing, removed);
            for root in bridge.template_surface.surface.tree.roots.clone() {
                let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                node.dirty.layout = true;
                node.dirty.input = true;
            }
            bridge.refresh_prepared_state_change().unwrap();
            assert!(bridge.settings_values_context().is_none());
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            assert!(bridge
                .refresh_settings_persistence_health(&health, Some(&retry))
                .unwrap());
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            assert!(bridge.settings_values_context().is_none());
            bridge.refresh_settings_values(&displayed).unwrap();
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            assert_eq!(pending_health(&mutations).1, retry);
            assert_eq!(
                mutations.authority().snapshot().generation(),
                displayed.generation()
            );
            mutations.shutdown().wait();
        }
    }
}

mod prepared_state_change {
    use super::*;

    #[test]
    fn successful_prepared_rebuild_preserves_only_the_original_settings_contexts() {
        let (_fixture, mutations) = pending_user_settings();
        let (health, retry) = pending_health(&mutations);
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                .unwrap();
        bridge
            .open_settings_window(
                guarded_preferences_tests::open_state(
                    &mutations,
                    "builtin|settings.category.editor",
                )
                .with_persistence_retry_context(Some(retry.clone())),
            )
            .unwrap();
        let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
        bridge.refresh_prepared_state_change().unwrap();
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert_eq!(
            bridge.settings_persistence_retry_context("user").unwrap(),
            retry
        );
        assert!(!bridge
            .refresh_settings_persistence_health(&health, Some(&retry))
            .unwrap());

        mutations.retry_pending(SettingsScope::User).unwrap();
        let (health, retry) = pending_health(&mutations);
        // Use the production notification order: prepare, then the shared rebuild.
        assert!(bridge.prepare_settings_persistence_health(&health).unwrap());
        assert!(bridge.settings_persistence_retry_context("user").is_err());
        bridge.refresh_prepared_state_change().unwrap();
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        // A prepared notification cannot manufacture a new retry target.
        assert!(bridge.settings_persistence_retry_context("user").is_err());
        assert!(bridge
            .refresh_settings_persistence_health(&health, Some(&retry))
            .unwrap());
        assert_eq!(bridge.settings_values_context(), Some(&displayed));
        assert_eq!(
            bridge.settings_persistence_retry_context("user").unwrap(),
            retry
        );
        assert_eq!(
            mutations.authority().snapshot().generation(),
            displayed.generation()
        );
        mutations.shutdown().wait();
    }

    #[test]
    fn failed_prepared_rebuild_revokes_settings_contexts_until_explicit_refresh() {
        for prepare_health in [false, true] {
            let (_fixture, mutations) = pending_user_settings();
            let (_, retry) = pending_health(&mutations);
            let mut bridge =
                BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
                    .unwrap();
            bridge
                .open_settings_window(
                    guarded_preferences_tests::open_state(
                        &mutations,
                        "builtin|settings.category.editor",
                    )
                    .with_persistence_retry_context(Some(retry.clone())),
                )
                .unwrap();
            let displayed = bridge.settings_mutation_context(EDITOR_LOCALE_KEY).unwrap();
            if prepare_health {
                mutations.retry_pending(SettingsScope::User).unwrap();
                let (health, _) = pending_health(&mutations);
                assert!(bridge.prepare_settings_persistence_health(&health).unwrap());
            }
            let (health, retry) = pending_health(&mutations);
            let missing = bridge.control_node_id("WorkbenchSceneTitle").unwrap();
            let removed = bridge
                .template_surface
                .surface
                .tree
                .nodes
                .remove(&missing)
                .unwrap();
            for root in bridge.template_surface.surface.tree.roots.clone() {
                let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                node.dirty.layout = true;
                node.dirty.input = true;
            }
            let error = bridge.refresh_prepared_state_change().unwrap_err();
            assert!(
                matches!(
                    &error,
                    BuiltinHostWindowTemplateBridgeError::ComponentizedWorkbench(
                        EditorWorkbenchTemplateSurfaceError::Tree(UiTreeError::MissingNode(id))
                    ) if *id == missing
                ),
                "prepare_health={prepare_health}: {error}"
            );
            assert!(bridge.settings_values_context().is_none());
            assert!(bridge.settings_mutation_context(EDITOR_LOCALE_KEY).is_err());
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            assert_eq!(pending_health(&mutations).1, retry);
            assert_eq!(
                mutations.authority().snapshot().generation(),
                displayed.generation()
            );

            bridge
                .template_surface
                .surface
                .tree
                .nodes
                .insert(missing, removed);
            for root in bridge.template_surface.surface.tree.roots.clone() {
                let node = bridge.template_surface.surface.tree.node_mut(root).unwrap();
                node.dirty.layout = true;
                node.dirty.input = true;
            }
            bridge.refresh_prepared_state_change().unwrap();
            assert!(bridge.settings_values_context().is_none());
            assert!(bridge.settings_persistence_retry_context("user").is_err());
            assert!(bridge
                .refresh_settings_persistence_health(&health, Some(&retry))
                .unwrap());
            assert!(bridge.settings_values_context().is_none());
            assert_eq!(
                bridge.settings_persistence_retry_context("user").unwrap(),
                retry
            );
            bridge.refresh_settings_values(&displayed).unwrap();
            assert_eq!(bridge.settings_values_context(), Some(&displayed));
            assert_eq!(pending_health(&mutations).1, retry);
            assert_eq!(
                mutations.authority().snapshot().generation(),
                displayed.generation()
            );
            mutations.shutdown().wait();
        }
    }
}
