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
