use super::*;

fn target(runtime_mode: RuntimeTargetMode) -> NativePluginArtifactTarget {
    NativePluginArtifactTarget::new(
        runtime_mode,
        crate::core::framework::project::ExportTargetPlatform::Windows,
    )
}

#[test]
fn only_enabled_native_selections_for_the_current_mode_are_resolved() {
    use crate::core::framework::project::ProjectPluginSelection;

    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin("editor", true, true)
                .with_packaging(ExportPackagingStrategy::NativeDynamic)
                .with_target_modes([RuntimeTargetMode::EditorHost]),
            ProjectPluginSelection::runtime_plugin("client", true, true)
                .with_packaging(ExportPackagingStrategy::NativeDynamic)
                .with_target_modes([RuntimeTargetMode::ClientRuntime]),
            ProjectPluginSelection::runtime_plugin("disabled", false, true)
                .with_packaging(ExportPackagingStrategy::NativeDynamic)
                .with_target_modes([RuntimeTargetMode::EditorHost]),
        ],
    };

    let active = active_native_selections(&manifest, &target(RuntimeTargetMode::EditorHost));

    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, "editor");
    assert!(active[0].required);
}

#[test]
fn required_admission_failures_block_ready_while_optional_failures_remain_reported() {
    let target = target(RuntimeTargetMode::EditorHost);
    let optional = NativePluginAdmission {
        target: target.clone(),
        policy_status: NativePluginPolicyStatus::Configured,
        installed_root: None,
        authority: NativePluginArtifactAuthority::deny_all(),
        outcomes: vec![NativePluginSelectionOutcome {
            plugin_id: "optional".into(),
            required: false,
            status: NativePluginSelectionStatus::MissingInstallation,
            detail: "native_plugin_installation_missing",
        }],
    };
    assert!(!optional.has_required_failures());
    assert_eq!(optional.outcomes().len(), 1);
    assert_eq!(
        optional.diagnostics(),
        [
            "native_plugin_selection target=EditorHost plugin_id=optional required=false status=MissingInstallation detail=native_plugin_installation_missing"
        ]
    );
    assert_eq!(optional.required_failure_diagnostic(), None);

    let required = NativePluginAdmission {
        target,
        policy_status: NativePluginPolicyStatus::Configured,
        installed_root: None,
        authority: NativePluginArtifactAuthority::deny_all(),
        outcomes: vec![
            NativePluginSelectionOutcome {
                plugin_id: "optional".into(),
                required: false,
                status: NativePluginSelectionStatus::MissingInstallation,
                detail: "native_plugin_installation_missing",
            },
            NativePluginSelectionOutcome {
                plugin_id: "required".into(),
                required: true,
                status: NativePluginSelectionStatus::MissingInstallation,
                detail: "native_plugin_installation_missing",
            },
        ],
    };
    assert!(required.has_required_failures());
    let reason = required
        .required_failure_diagnostic()
        .expect("required selection must block Ready");
    assert!(reason.contains("required:MissingInstallation:native_plugin_installation_missing"));
    assert!(reason.contains(
        "native_plugin_selection target=EditorHost plugin_id=optional required=false status=MissingInstallation detail=native_plugin_installation_missing"
    ));
    assert!(reason.contains(
        "native_plugin_selection target=EditorHost plugin_id=required required=true status=MissingInstallation detail=native_plugin_installation_missing"
    ));
}

#[test]
fn missing_explicit_install_selection_is_reported_and_blocks_required_plugins() {
    let admission = NativePluginAdmission {
        target: target(RuntimeTargetMode::EditorHost),
        policy_status: NativePluginPolicyStatus::Configured,
        installed_root: None,
        authority: NativePluginArtifactAuthority::deny_all(),
        outcomes: vec![
            NativePluginSelectionOutcome {
                plugin_id: "optional".into(),
                required: false,
                status: NativePluginSelectionStatus::MissingSelection,
                detail: "native_plugin_installed_selection_missing",
            },
            NativePluginSelectionOutcome {
                plugin_id: "required".into(),
                required: true,
                status: NativePluginSelectionStatus::MissingSelection,
                detail: "native_plugin_installed_selection_missing",
            },
        ],
    };

    assert!(admission.has_required_failures());
    let diagnostic = admission
        .required_failure_diagnostic()
        .expect("required plugin without a host install selection must block Ready");
    assert!(
        diagnostic.contains("required:MissingSelection:native_plugin_installed_selection_missing")
    );
    assert!(diagnostic.contains("plugin_id=optional required=false status=MissingSelection"));
}

#[test]
fn selected_install_lookup_requires_exact_revision_and_artifact_digest() {
    let selection = NativePluginInstalledSelection {
        target: NativePluginArtifactTarget::new(
            RuntimeTargetMode::EditorHost,
            crate::core::framework::project::ExportTargetPlatform::Windows,
        ),
        plugin_id: "studio.physics".into(),
        identity_digest: "a".repeat(64),
        package_id: "3298de17-8f1a-4e04-a28e-55c1ff43f2d8".into(),
        release_revision: "7".into(),
        artifact_digest: "b".repeat(64),
    };
    let package = InstalledPackage {
        operation_id: "9e8928ea-aaca-48ba-a49b-604deca14ee8".into(),
        package_id: selection.package_id.clone(),
        version: "1.2.3".into(),
        release_revision: selection.release_revision.clone(),
        artifact_digest: selection.artifact_digest.clone(),
        slot: format!(
            "slots/{}/{}",
            selection.package_id, selection.artifact_digest
        ),
        files: std::collections::BTreeMap::new(),
    };
    let inventory = PackageInventory {
        schema_version: 1,
        revision: "1".into(),
        packages: vec![package.clone()],
    };

    assert!(matches!(
        selected_install_from_inventory(&selection, &inventory),
        SelectedInstallLookup::Matched(_)
    ));

    let mut stale = inventory.clone();
    stale.packages[0].artifact_digest = "c".repeat(64);
    assert!(matches!(
        selected_install_from_inventory(&selection, &stale),
        SelectedInstallLookup::Stale
    ));

    let missing = PackageInventory {
        schema_version: 1,
        revision: "0".into(),
        packages: Vec::new(),
    };
    assert!(matches!(
        selected_install_from_inventory(&selection, &missing),
        SelectedInstallLookup::Missing
    ));
}
