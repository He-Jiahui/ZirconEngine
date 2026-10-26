use super::*;
use crate::ui::host::EditorPluginStatus;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;

fn fixture_report(count: usize) -> Arc<EditorPluginStatusReport> {
    Arc::new(EditorPluginStatusReport {
        enablement: None,
        plugins: ["vendor.first", "vendor.second"]
            .into_iter()
            .map(|owner| EditorPluginStatus {
                plugin_id: owner.into(),
                display_name: owner.into(),
                package_source: "native".into(),
                package_metadata: None,
                load_state: "manifest only".into(),
                enabled: true,
                required: false,
                target_modes: vec![],
                packaging: ExportPackagingStrategy::NativeDynamic,
                runtime_crate: None,
                editor_crate: None,
                runtime_capabilities: vec![],
                editor_capabilities: vec![],
                diagnostics: vec![],
                optional_features: (0..count)
                    .map(|index| EditorPluginFeatureStatus {
                        id: format!("shared.feature.{index}"),
                        display_name: format!("Feature {index}"),
                        owner_plugin_id: owner.into(),
                        enabled: index == 0,
                        required: index == 0,
                        available: index != 0,
                        target_modes: vec![],
                        packaging: ExportPackagingStrategy::NativeDynamic,
                        runtime_crate: None,
                        editor_crate: None,
                        provided_capabilities: vec![],
                        dependencies: vec![],
                        diagnostics: if index == 0 {
                            vec!["resolution blocked".into()]
                        } else {
                            vec![]
                        },
                    })
                    .collect(),
            })
            .collect(),
        diagnostics: vec![],
    })
}

fn view(
    state: &mut ModulePluginDetailsState,
    report: &Arc<EditorPluginStatusReport>,
) -> Rc<ModulePluginDetailsViewData> {
    state
        .project("project-a", report, "en", str::to_owned)
        .expect("open details")
}

fn recovery_report(
    recovery: PluginEnablementRecovery,
    owner: &str,
) -> Arc<EditorPluginStatusReport> {
    use crate::ui::host::{
        PluginEnablementApplied, PluginEnablementDurable, PluginEnablementPhase,
    };
    let mut report = (*fixture_report(2)).clone();
    let reverted = recovery == PluginEnablementRecovery::RestartRequired;
    report.enablement = Some(PluginEnablementOperationReport {
        operation_id: "details-recovery-fixture".into(),
        project_root: "fixture-project".into(),
        project_instance: "fixture-instance".into(),
        session_generation: 7,
        plugin_id: owner.into(),
        requested_enabled: false,
        expected_revision: Some(zircon_runtime::core::resource::io::SourceRevision::Missing),
        committed_revision: None,
        phase: if reverted {
            PluginEnablementPhase::Complete
        } else {
            PluginEnablementPhase::Reconcile
        },
        durable: PluginEnablementDurable::Durable,
        applied: PluginEnablementApplied::NotApplied,
        recovery,
        primary_cause: None,
        rollback_cause: None,
        detail: None,
        recovery_admission: None,
    });
    Arc::new(report)
}

#[test]
fn report_refresh_restores_feature_focus_when_apply_is_no_longer_actionable() {
    for change in ["removed", "required", "foreign-owner"] {
        let original = fixture_report(3);
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &original);
        assert!(state.open("vendor.first", &original));
        assert!(state.select(1, "shared.feature.1", &original));
        state.focus = "apply".into();
        let ready = view(&mut state, &original);
        assert!(!ready.intent_id.is_empty());
        assert_eq!(state.focus, "apply");

        let mut updated = (*original).clone();
        let features = &mut updated.plugins[0].optional_features;
        match change {
            "removed" => {
                features.remove(1);
            }
            "required" => {
                features[1].enabled = true;
                features[1].required = true;
            }
            "foreign-owner" => features[1].owner_plugin_id = "vendor.second".into(),
            _ => unreachable!(),
        }
        let updated = Arc::new(updated);
        let refreshed = view(&mut state, &updated);
        assert_eq!(state.focus, "features", "{change}");
        assert_eq!(state.package_selection, "vendor.first");
        assert_eq!(
            refreshed.selected_feature_id,
            if change == "removed" {
                ""
            } else {
                "shared.feature.1"
            }
        );
        assert!(refreshed.action_id.is_empty() && refreshed.intent_id.is_empty());
        assert!(state
            .take_action("project-a", &updated, &ready.intent_id)
            .is_none());
        let generation = state.generation;
        assert!(Rc::ptr_eq(&refreshed, &view(&mut state, &updated)));
        assert_eq!(state.generation, generation);
        assert_eq!(updated.plugins[1], original.plugins[1]);
        assert_eq!(updated.enablement, original.enablement);

        let index = updated.plugins[0]
            .optional_features
            .iter()
            .position(|feature| feature.id == "shared.feature.2")
            .unwrap();
        assert!(state.select(index, "shared.feature.2", &updated));
        let next = view(&mut state, &updated);
        assert!(!next.intent_id.is_empty());
        assert_eq!(
            state.take_action("project-a", &updated, &next.intent_id),
            Some(next.action_id.clone())
        );
    }
}

#[test]
fn empty_feature_report_restores_back_focus_and_retires_detail_intent() {
    for focus in ["features", "apply", "previous", "next"] {
        let original = fixture_report(19);
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &original);
        assert!(state.open("vendor.first", &original));
        assert!(state.turn_page(true, &original));
        assert!(state.select(1, "shared.feature.9", &original));
        state.focus = focus.into();
        let ready = view(&mut state, &original);
        assert_eq!(state.focus, focus);
        assert!(!ready.intent_id.is_empty());

        let mut empty = (*original).clone();
        empty.plugins[0].optional_features.clear();
        let empty = Arc::new(empty);
        let refreshed = view(&mut state, &empty);
        assert_eq!(state.focus, "back", "{focus}");
        assert_eq!(refreshed.page, 0);
        assert!(refreshed.rows.is_empty() && refreshed.selected_feature_id.is_empty());
        assert!(refreshed.action_id.is_empty() && refreshed.intent_id.is_empty());
        assert!(state
            .take_action("project-a", &empty, &ready.intent_id)
            .is_none());
        assert!(Rc::ptr_eq(&refreshed, &view(&mut state, &empty)));
        assert_eq!(empty.plugins[1], original.plugins[1]);

        state.back();
        assert_eq!(state.package_selection, "vendor.first");
        assert_eq!(state.focus, "package");
        assert!(state.owner.is_none() && state.feature.is_none() && state.intent.is_none());
    }
}

#[test]
fn report_refresh_restores_focus_when_pagination_controls_disappear() {
    for focus in ["previous", "next"] {
        let original = fixture_report(19);
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &original);
        assert!(state.open("vendor.first", &original));
        assert!(state.turn_page(true, &original));
        assert!(state.select(1, "shared.feature.9", &original));
        state.focus = focus.into();
        let ready = view(&mut state, &original);
        assert_eq!(state.focus, focus);

        let mut updated = (*original).clone();
        let features = &mut updated.plugins[0].optional_features;
        if focus == "previous" {
            *features = features.split_off(FEATURES_PER_PAGE);
        } else {
            features.truncate(13);
        }
        let updated = Arc::new(updated);
        let refreshed = view(&mut state, &updated);
        assert_eq!(state.focus, "features", "{focus}");
        assert_eq!(refreshed.page, usize::from(focus == "next"));
        assert_eq!(refreshed.selected_feature_id, "shared.feature.9");
        assert_eq!(refreshed.action_id, ready.action_id);
        assert_ne!(refreshed.intent_id, ready.intent_id);
        assert!(state
            .take_action("project-a", &updated, &ready.intent_id)
            .is_none());
        assert!(Rc::ptr_eq(&refreshed, &view(&mut state, &updated)));
        assert_eq!(updated.plugins[1], original.plugins[1]);
    }
}

#[test]
fn recovery_retires_detail_apply_intent_and_restores_feature_focus() {
    for recovery in [
        PluginEnablementRecovery::PendingRuntimeApply,
        PluginEnablementRecovery::RuntimeRepairRequired,
        PluginEnablementRecovery::FileRecoveryRequired,
        PluginEnablementRecovery::RestartRequired,
    ] {
        let initial = fixture_report(2);
        let mut state = ModulePluginDetailsState::default();
        state.reconcile("project-a", &initial);
        assert!(state.open("vendor.first", &initial));
        assert!(state.select(1, "shared.feature.1", &initial));
        state.focus = "apply".into();
        let ready = view(&mut state, &initial);
        assert!(!ready.intent_id.is_empty());
        let report = recovery_report(recovery, "vendor.first");
        let operation = report.enablement.clone();
        let blocked = view(&mut state, &report);
        assert!(blocked.action_id.is_empty() && blocked.intent_id.is_empty());
        assert_eq!(blocked.selected_feature_id, "shared.feature.1");
        assert_eq!(blocked.selected_states, ready.selected_states);
        assert_eq!(blocked.rows[1].desired, ready.rows[1].desired);
        assert!(blocked
            .selected_reason
            .contains(&operation.as_ref().unwrap().status_message()));
        assert_eq!(state.focus, "features");
        assert!(state
            .take_action("project-a", &report, &ready.intent_id)
            .is_none());
        assert_eq!(report.enablement, operation);
        assert!(Rc::ptr_eq(&blocked, &view(&mut state, &report)));
    }
}

#[test]
fn clearing_recovery_reenables_only_a_fresh_qualified_detail_intent() {
    let blocked_report = recovery_report(
        PluginEnablementRecovery::PendingRuntimeApply,
        "vendor.first",
    );
    let mut state = ModulePluginDetailsState::default();
    state.reconcile("project-a", &blocked_report);
    assert!(state.open("vendor.first", &blocked_report));
    assert!(state.select(1, "shared.feature.1", &blocked_report));
    assert!(view(&mut state, &blocked_report).action_id.is_empty());
    let ready_report = fixture_report(2);
    let ready = view(&mut state, &ready_report);
    assert_eq!(ready.selected_feature_id, "shared.feature.1");
    assert_eq!(
        ready.action_id,
        "workbench.plugin.feature.enable.#12:vendor.first.shared.feature.1"
    );
    assert!(!ready.intent_id.is_empty());
    assert_eq!(
        state.take_action("project-a", &ready_report, &ready.intent_id),
        Some(ready.action_id.clone())
    );
    assert!(state
        .take_action("project-a", &ready_report, &ready.intent_id)
        .is_none());
}

#[test]
fn detail_recovery_gate_preserves_other_plugin_actions_and_feature_diagnostics() {
    let other_report = recovery_report(
        PluginEnablementRecovery::PendingRuntimeApply,
        "vendor.second",
    );
    let mut state = ModulePluginDetailsState::default();
    state.reconcile("project-a", &other_report);
    assert!(state.open("vendor.first", &other_report));
    assert!(state.select(1, "shared.feature.1", &other_report));
    assert!(!view(&mut state, &other_report).action_id.is_empty());
    let own_report = recovery_report(PluginEnablementRecovery::RestartRequired, "vendor.first");
    view(&mut state, &own_report);
    assert!(state.select(0, "shared.feature.0", &own_report));
    let blocked = view(&mut state, &own_report);
    assert!(blocked.selected_reason.contains("resolution blocked"));
    assert!(blocked.selected_reason.contains("pending change reverted"));
    assert!(!blocked.selected_reason.contains("change saved"));
    assert!(blocked.action_id.is_empty());
    assert!(blocked.rows[0].required && blocked.rows[0].blocked);
}

#[test]
fn feature_pages_cover_all_features_with_honest_status_authority() {
    let report = fixture_report(19);
    let mut state = ModulePluginDetailsState::default();
    state.reconcile("project-a", &report);
    assert!(state.open("vendor.first", &report));
    let mut identities = Vec::new();
    loop {
        let page = view(&mut state, &report);
        assert!(page.rows.len() <= FEATURES_PER_PAGE);
        assert!(page.rows.iter().all(|row| row.effective == "unknown"
            && row.label.contains("desired:")
            && row.label.contains("effective: unknown")
            && row.label.contains("required:")
            && row.label.contains("blocked:")));
        identities.extend(page.rows.iter().map(|row| row.feature_id.clone()));
        if !state.turn_page(true, &report) {
            break;
        }
    }
    assert_eq!(
        identities,
        (0..19)
            .map(|index| format!("shared.feature.{index}"))
            .collect::<Vec<_>>()
    );
    assert!(state.turn_page(false, &report));
    assert!(state.turn_page(false, &report));
    assert!(state.select(0, "shared.feature.0", &report));
    let selected = view(&mut state, &report);
    assert!(selected.rows[0].desired && selected.rows[0].required && selected.rows[0].blocked);
    assert_eq!(selected.selected_reason, "resolution blocked");
    assert!(selected.action_id.is_empty());
    let empty = fixture_report(0);
    state.reconcile("project-a", &empty);
    let empty_view = view(&mut state, &empty);
    assert!(empty_view.rows.is_empty());
    assert_eq!(empty_view.page, 0);
    assert!(!state.turn_page(true, &empty));
}

#[test]
fn selection_survives_reorder_and_resets_on_owner_or_project_change() {
    let original = fixture_report(19);
    let mut state = ModulePluginDetailsState::default();
    state.reconcile("project-a", &original);
    state.open("vendor.first", &original);
    assert!(state.turn_page(true, &original));
    assert!(state.select(1, "shared.feature.9", &original));
    let mut reordered = (*original).clone();
    reordered.plugins.reverse();
    reordered.plugins[1].optional_features.reverse();
    let reordered = Arc::new(reordered);
    let page = view(&mut state, &reordered);
    assert_eq!(page.plugin_id, "vendor.first");
    assert_eq!(page.selected_feature_id, "shared.feature.9");
    assert_eq!(page.page, 1);
    let stable = view(&mut state, &reordered);
    assert!(Rc::ptr_eq(&page, &stable));
    let localized = state
        .project("project-a", &reordered, "zh-CN", |key| format!("中:{key}"))
        .unwrap();
    assert!(!Rc::ptr_eq(&page, &localized));
    assert!(localized.selected_states.contains("中:unknown"));
    state.open("vendor.second", &reordered);
    assert!(view(&mut state, &reordered).selected_feature_id.is_empty());
    state.back();
    assert_eq!(state.package_selection, "vendor.second");
    assert_eq!(state.focus, "package");
    state.open("vendor.second", &reordered);
    assert!(state
        .project("project-b", &reordered, "en", str::to_owned)
        .is_none());
    assert!(state.feature.is_none() && state.intent.is_none() && state.cached.is_none());
    state.open("vendor.first", &reordered);
    let removed = Arc::new(EditorPluginStatusReport::default());
    assert!(state
        .project("project-b", &removed, "en", str::to_owned)
        .is_none());
}

#[test]
fn selected_feature_actions_keep_qualified_owner_and_reject_stale_targets() {
    let report = fixture_report(10);
    let mut state = ModulePluginDetailsState::default();
    state.reconcile("project-a", &report);
    state.open("vendor.first", &report);
    state.select(1, "shared.feature.1", &report);
    let first = view(&mut state, &report);
    assert_eq!(
        first.action_id,
        "workbench.plugin.feature.enable.#12:vendor.first.shared.feature.1"
    );
    state.open("vendor.second", &report);
    assert!(state
        .take_action("project-a", &report, &first.intent_id)
        .is_none());
    state.select(1, "shared.feature.1", &report);
    let second = view(&mut state, &report);
    assert_eq!(
        second.action_id,
        "workbench.plugin.feature.enable.#13:vendor.second.shared.feature.1"
    );
    assert_eq!(
        state.take_action("project-a", &report, &second.intent_id),
        Some(second.action_id.clone())
    );
    assert!(state
        .take_action("project-a", &report, &second.intent_id)
        .is_none());
    let refreshed = view(&mut state, &report);
    state.turn_page(true, &report);
    assert!(state
        .take_action("project-a", &report, &refreshed.intent_id)
        .is_none());
    state.turn_page(false, &report);
    state.select(1, "shared.feature.1", &report);
    let previous = view(&mut state, &report);
    let mut changed = (*report).clone();
    changed.plugins[1].optional_features[1].enabled = true;
    let changed = Arc::new(changed);
    assert!(state
        .take_action("project-a", &changed, &previous.intent_id)
        .is_none());
    assert!(view(&mut state, &changed)
        .action_id
        .starts_with("workbench.plugin.feature.disable."));
}

#[test]
fn closed_package_selection_recovers_when_report_removes_the_back_target() {
    let original = fixture_report(19);
    let mut state = ModulePluginDetailsState::default();
    state.reconcile("project-a", &original);
    assert!(state.open("vendor.second", &original));
    assert!(state.select(1, "shared.feature.1", &original));
    assert!(!view(&mut state, &original).intent_id.is_empty());
    state.back();
    assert_eq!(state.package_selection, "vendor.second");

    let mut removed = (*original).clone();
    removed
        .plugins
        .retain(|plugin| plugin.plugin_id != "vendor.second");
    let removed = Arc::new(removed);
    assert!(state
        .project("project-a", &removed, "en", |_| panic!("closed details"))
        .is_none());
    assert_eq!(state.package_selection, "vendor.first");
    assert_eq!(state.focus, "package");
    assert!(state.owner.is_none() && state.feature.is_none());
    assert_eq!(state.page, 0);
    assert!(state.cached.is_none() && state.intent.is_none());
    let generation = state.generation;
    for _ in 0..100 {
        assert!(state
            .project("project-a", &removed, "en", |_| panic!("closed details"))
            .is_none());
        assert_eq!(state.generation, generation);
        assert!(state.cached.is_none() && state.intent.is_none());
    }

    let mut reordered = (*original).clone();
    reordered.plugins.reverse();
    let reordered = Arc::new(reordered);
    state.reconcile("project-a", &reordered);
    assert_eq!(state.package_selection, "vendor.first");
    let owner = state.package_selection.clone();
    assert!(state.open(&owner, &reordered));
    assert_eq!(view(&mut state, &reordered).plugin_id, "vendor.first");
    state.back();

    let empty = Arc::new(EditorPluginStatusReport::default());
    assert!(state
        .project("project-a", &empty, "en", |_| panic!("closed details"))
        .is_none());
    assert!(state.package_selection.is_empty() && state.owner.is_none());
    assert_eq!(state.focus, "package");
    assert!(state.cached.is_none() && state.intent.is_none());
    state.reconcile("project-a", &original);
    assert!(state.package_selection.is_empty());
    assert!(state.open(&original.plugins[0].plugin_id, &original));
    assert_eq!(view(&mut state, &original).plugin_id, "vendor.first");
}

#[test]
fn closed_details_keep_the_existing_summary_generation_without_feature_pages() {
    let report = fixture_report(2_048);
    let mut summary = super::super::cache::ModulePluginPaneProjectionCache::default();
    let mut state = ModulePluginDetailsState::default();
    let first = summary.get_or_build(
        &report,
        super::super::pane_data::module_plugins_pane_from_status_report,
    );
    for _ in 0..100 {
        let pane =
            summary.get_or_build(&report, |_| panic!("summary must reuse its published Arc"));
        assert!(first.plugins.shares_values_with(&pane.plugins));
        assert!(state
            .project("project-a", &report, "en", |_| panic!(
                "closed detail projection"
            ))
            .is_none());
        assert!(state.cached.is_none() && state.intent.is_none());
    }
}
