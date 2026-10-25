use crate::ui::host::{EditorPluginStatus, EditorPluginStatusReport};
use crate::ui::layouts::windows::workbench_host_window::ModulePluginStatusViewData;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;

use super::super::rows::{
    module_plugin_action_id, module_plugin_feature_action, module_plugin_optional_feature_summary,
    module_plugin_primary_action, packaging_label, target_mode_label,
};

pub(super) fn module_plugin_status_rows(
    report: &EditorPluginStatusReport,
) -> Vec<ModulePluginStatusViewData> {
    let mut rows = Vec::with_capacity(report.plugins.len());
    rows.extend(report.plugins.iter().map(module_plugin_status_row));
    if let Some(operation) = report.enablement.as_ref() {
        use crate::ui::host::PluginEnablementRecovery as Recovery;
        let index = rows
            .iter()
            .position(|r| r.plugin_id.as_str() == operation.plugin_id)
            .unwrap_or_else(|| {
                rows.push(ModulePluginStatusViewData {
                    plugin_id: operation.plugin_id.clone().into(),
                    display_name: "Pending project plugin change".into(),
                    package_source: "Project".into(),
                    package_metadata: None,
                    load_state: "".into(),
                    enabled: false,
                    required: false,
                    target_modes: "".into(),
                    packaging: "".into(),
                    runtime_crate: "".into(),
                    editor_crate: "".into(),
                    runtime_capabilities: "".into(),
                    editor_capabilities: "".into(),
                    optional_features: "".into(),
                    feature_action_label: "".into(),
                    feature_action_id: "".into(),
                    diagnostics: "".into(),
                    primary_action_label: "".into(),
                    primary_action_id: "".into(),
                    packaging_action_label: "".into(),
                    packaging_action_id: "".into(),
                    target_modes_action_label: "".into(),
                    target_modes_action_id: "".into(),
                    unload_action_label: "".into(),
                    unload_action_id: "".into(),
                    hot_reload_action_label: "".into(),
                    hot_reload_action_id: "".into(),
                });
                rows.len() - 1
            });
        let row = &mut rows[index];
        let action = |verb| {
            format!(
                "workbench.plugin.enablement.{verb}:{}",
                operation.operation_id
            )
        };
        row.diagnostics = operation.status_message().into();
        if operation.recovery != Recovery::None {
            row.packaging_action_id = "".into();
            row.target_modes_action_id = "".into();
            row.feature_action_id = "".into();
            row.unload_action_id = "".into();
            row.hot_reload_action_id = "".into();
        }
        match operation.recovery {
            Recovery::None => {}
            Recovery::PendingRuntimeApply => {
                row.load_state = "Saved; runtime changes pending".into();
                if operation.recovery_admission.is_some() {
                    row.primary_action_label = "Revert pending change".into();
                    row.primary_action_id = action("revert").into();
                } else {
                    row.primary_action_label = "Retry runtime apply".into();
                    row.primary_action_id = action("retry").into();
                    row.packaging_action_label = "Revert pending change".into();
                    row.packaging_action_id = action("revert").into();
                }
            }
            Recovery::RuntimeRepairRequired => {
                row.load_state = "Saved; runtime repair required".into();
                if operation.recovery_admission.is_some() {
                    row.primary_action_label = "Revert pending change".into();
                    row.primary_action_id = action("revert").into();
                } else {
                    row.primary_action_label = "Close project, then reopen to resolve".into();
                    row.primary_action_id = "".into();
                }
            }
            Recovery::FileRecoveryRequired => {
                row.load_state = "Project file recovery required".into();
                row.primary_action_label = "Recover project files".into();
                row.primary_action_id = if operation.recovery_admission.is_some() {
                    action("recover").into()
                } else {
                    "".into()
                };
            }
            Recovery::RestartRequired => {
                row.load_state = "Pending change reverted; project can be reopened".into();
                row.primary_action_label = "Open project".into();
                row.primary_action_id = action("open").into();
            }
        }
    }

    rows
}

fn module_plugin_status_row(plugin: &EditorPluginStatus) -> ModulePluginStatusViewData {
    let primary_action =
        module_plugin_primary_action(&plugin.plugin_id, plugin.enabled, plugin.required);
    let packaging_action_label = format!("Cycle {}", packaging_label(plugin.packaging));
    let packaging_action_id =
        module_plugin_action_id("workbench.plugin.packaging.next", &plugin.plugin_id);
    let target_modes_action_id =
        module_plugin_action_id("workbench.plugin.target_modes.next", &plugin.plugin_id);
    let unload_action_id = module_plugin_action_id("workbench.plugin.unload", &plugin.plugin_id);
    let hot_reload_action_id =
        module_plugin_action_id("workbench.plugin.hot_reload", &plugin.plugin_id);
    let feature_action = module_plugin_feature_action(&plugin.optional_features);

    ModulePluginStatusViewData {
        plugin_id: plugin.plugin_id.clone().into(),
        display_name: plugin.display_name.clone().into(),
        package_source: plugin.package_source.clone().into(),
        package_metadata: plugin.package_metadata.clone(),
        load_state: plugin.load_state.clone().into(),
        enabled: plugin.enabled,
        required: plugin.required,
        target_modes: target_mode_summary(&plugin.target_modes).into(),
        packaging: packaging_label(plugin.packaging).into(),
        runtime_crate: plugin.runtime_crate.clone().unwrap_or_default().into(),
        editor_crate: plugin.editor_crate.clone().unwrap_or_default().into(),
        runtime_capabilities: plugin.runtime_capabilities.join(", ").into(),
        editor_capabilities: plugin.editor_capabilities.join(", ").into(),
        optional_features: module_plugin_optional_feature_summary(&plugin.optional_features).into(),
        feature_action_label: feature_action.0.into(),
        feature_action_id: feature_action.1.into(),
        diagnostics: plugin.diagnostics.join("\n").into(),
        primary_action_label: primary_action.0.into(),
        primary_action_id: primary_action.1.into(),
        packaging_action_label: packaging_action_label.into(),
        packaging_action_id: packaging_action_id.into(),
        target_modes_action_label: "Cycle targets".into(),
        target_modes_action_id: target_modes_action_id.into(),
        unload_action_label: "Unload".into(),
        unload_action_id: unload_action_id.into(),
        hot_reload_action_label: "Hot Reload".into(),
        hot_reload_action_id: hot_reload_action_id.into(),
    }
}

fn target_mode_summary(target_modes: &[RuntimeTargetMode]) -> String {
    let mut summary = String::with_capacity(target_modes.len().saturating_mul("editor, ".len()));
    for (index, mode) in target_modes.iter().enumerate() {
        if index != 0 {
            summary.push_str(", ");
        }
        summary.push_str(target_mode_label(mode));
    }
    summary
}

#[cfg(test)]
#[path = "view_rows/tests/target_mode_join.rs"]
mod target_mode_join_tests;

#[cfg(test)]
mod enablement_state_tests {
    use super::*;
    use crate::ui::host::*;
    #[test]
    fn typed_operation_projects_reachable_recovery_actions_without_a_live_plugin_row() {
        for (recovery, label, verb) in [
            (
                PluginEnablementRecovery::PendingRuntimeApply,
                "Saved; runtime changes pending",
                Some("retry"),
            ),
            (
                PluginEnablementRecovery::RuntimeRepairRequired,
                "Saved; runtime repair required",
                None,
            ),
            (
                PluginEnablementRecovery::FileRecoveryRequired,
                "Project file recovery required",
                None,
            ),
            (
                PluginEnablementRecovery::RestartRequired,
                "Pending change reverted; project can be reopened",
                Some("open"),
            ),
        ] {
            let operation = PluginEnablementOperationReport {
                operation_id: "typed-op".into(),
                project_root: std::path::PathBuf::from("E:/cargo-targets/zircon-local/not-created"),
                project_instance: "".into(),
                session_generation: 0,
                plugin_id: "not-loaded".into(),
                requested_enabled: true,
                expected_revision: None,
                committed_revision: None,
                phase: PluginEnablementPhase::Reconcile,
                durable: PluginEnablementDurable::Durable,
                applied: PluginEnablementApplied::NotApplied,
                recovery,
                primary_cause: None,
                rollback_cause: None,
                detail: None,
                recovery_admission: None,
            };
            let report = EditorPluginStatusReport {
                plugins: vec![],
                diagnostics: vec![],
                enablement: Some(operation),
            };
            let rows = module_plugin_status_rows(&report);
            let row = rows
                .iter()
                .find(|row| row.plugin_id.as_str() == "not-loaded")
                .unwrap();
            assert_eq!(row.load_state.as_str(), label);
            assert_eq!(row.package_metadata, None);
            assert!(!row.enabled);
            assert_eq!(
                row.primary_action_id.as_str(),
                verb.map(|v| format!("workbench.plugin.enablement.{v}:typed-op"))
                    .unwrap_or_default()
            );
            assert!(row.unload_action_id.is_empty() && row.hot_reload_action_id.is_empty());
        }
    }

    #[test]
    fn package_metadata_stays_with_plugin_identity_and_captured_report() {
        let metadata = EditorPluginPackageMetadata {
            version: "1.2.3".into(),
            category: "Rendering".into(),
            description: "真实清单描述 / manifest description".into(),
            maturity: zircon_runtime::plugin::PluginMaturity::Beta,
        };
        let status = |id: &str, package_metadata| EditorPluginStatus {
            plugin_id: id.into(),
            display_name: "Same translated display name".into(),
            package_source: "native".into(),
            package_metadata,
            load_state: "loaded".into(),
            enabled: true,
            required: false,
            target_modes: vec![],
            packaging:
                zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic,
            runtime_crate: None,
            editor_crate: None,
            runtime_capabilities: vec![],
            editor_capabilities: vec![],
            optional_features: vec![],
            diagnostics: vec![],
        };
        let mut report = EditorPluginStatusReport {
            plugins: vec![
                status("with-package", Some(metadata.clone())),
                status("missing", None),
            ],
            diagnostics: vec![],
            enablement: None,
        };
        let first = module_plugin_status_rows(&report);
        assert_eq!(first[0].package_metadata.as_ref(), Some(&metadata));
        assert_eq!(first[1].package_metadata, None);

        report.plugins.reverse();
        report.plugins[1].package_metadata = None;
        report.plugins[1].load_state = "not loaded".into();
        let next = module_plugin_status_rows(&report);
        assert_eq!(next[0].plugin_id.as_str(), "missing");
        assert_eq!(next[1].plugin_id.as_str(), "with-package");
        assert!(next.iter().all(|row| row.package_metadata.is_none()));
        assert_eq!(first[0].package_metadata.as_ref(), Some(&metadata));
        assert_eq!(first[0].load_state.as_str(), "loaded");
        assert_eq!(next[1].load_state.as_str(), "not loaded");
    }
}
