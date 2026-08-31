use std::collections::BTreeMap;

use zircon_runtime::plugin::native::NativePluginEditorCommandBinding;
use zircon_runtime::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::plugin::sdk::lifecycle::{
    EditorPluginLifecycleEvent, EditorPluginLifecycleRecord, EditorPluginLifecycleReport,
    EditorPluginLifecycleStage,
};
use crate::core::plugin::EditorPluginRegistrationReport;

use super::super::package_projection::editor_capabilities_for_package;

pub(super) fn package_declares_editor_contribution(package: &PluginPackageManifest) -> bool {
    package
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Editor)
        || !editor_capabilities_for_package(package).is_empty()
}

pub(super) fn native_editor_registration_from_package(
    package_manifest: PluginPackageManifest,
    extensions: EditorExtensionRegistry,
    native_command_bindings: BTreeMap<EditorOperationPath, NativePluginEditorCommandBinding>,
    mut diagnostics: Vec<String>,
) -> EditorPluginRegistrationReport {
    diagnostics.sort();
    diagnostics.dedup();
    let capabilities = editor_capabilities_for_package(&package_manifest);
    let lifecycle = native_package_lifecycle_report(&package_manifest);
    EditorPluginRegistrationReport {
        package_manifest: editor_only_package_manifest(package_manifest),
        capabilities,
        extensions,
        lifecycle,
        successful_lifecycle_stages: Vec::new(),
        failed_lifecycle_stages: Vec::new(),
        runtime_event_consumers:
            crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry::default(),
        native_command_bindings,
        diagnostics,
    }
}

fn editor_only_package_manifest(
    mut package_manifest: PluginPackageManifest,
) -> PluginPackageManifest {
    package_manifest
        .modules
        .retain(|module| module.kind == PluginModuleKind::Editor);
    package_manifest
}

fn native_package_lifecycle_report(
    package_manifest: &PluginPackageManifest,
) -> EditorPluginLifecycleReport {
    let mut report = EditorPluginLifecycleReport::default();
    for stage in [
        EditorPluginLifecycleStage::Loaded,
        EditorPluginLifecycleStage::Enabled,
    ] {
        report.record(EditorPluginLifecycleRecord::new(
            package_manifest.id.clone(),
            EditorPluginLifecycleEvent::new(stage),
        ));
    }
    report
}
