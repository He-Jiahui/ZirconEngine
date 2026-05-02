use std::{collections::HashSet, path::Path};
use zircon_runtime::asset::project::ProjectManifest;

use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::NativePluginLoader, plugin::PluginModuleKind,
    plugin::PluginPackageManifest, plugin::RuntimePluginCatalog, RuntimeTargetMode,
};

use super::super::super::editor_manager::EditorManager;
use super::super::package_projection::{
    editor_capabilities_for_package, module_crate, runtime_capabilities_for_package,
};
use super::super::reports::{
    EditorPluginFeatureStatus, EditorPluginStatus, EditorPluginStatusReport,
};
use super::builtin::{
    available_capabilities_for_feature_status, blocked_feature_diagnostic_map,
    optional_feature_statuses,
};
use super::native_load_state::native_load_state;

impl EditorManager {
    pub fn native_plugin_status_report(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
    ) -> EditorPluginStatusReport {
        let native_report =
            NativePluginLoader.load_discovered_all(self.plugin_directory(project_root));
        let native_packages = native_report.package_manifests();
        let mut status_report = self.plugin_status_report(manifest);
        let status_target = RuntimeTargetMode::EditorHost;
        let native_runtime_registrations = native_report.runtime_plugin_registration_reports();
        let status_runtime_catalog = RuntimePluginCatalog::from_registration_reports(
            self.runtime_plugin_catalog()
                .registrations()
                .iter()
                .cloned()
                .chain(native_runtime_registrations),
            [],
        );
        let completed_plugins = status_runtime_catalog.complete_project_manifest(&manifest.plugins);
        let feature_report =
            status_runtime_catalog.feature_dependency_report(&completed_plugins, status_target);
        let available_feature_ids = feature_report
            .available_features
            .iter()
            .cloned()
            .collect::<HashSet<_>>();
        let enabled_plugins = completed_plugins
            .enabled_for_target(status_target)
            .map(|selection| selection.id.clone())
            .collect::<HashSet<_>>();
        let mut status_packages = status_runtime_catalog.package_manifests();
        status_packages.extend(native_packages.iter().cloned());
        let available_capabilities = available_capabilities_for_feature_status(
            &status_packages,
            &enabled_plugins,
            &available_feature_ids,
            status_target,
        );
        let blocked_feature_diagnostics =
            blocked_feature_diagnostic_map(&feature_report.blocked_features);
        status_report
            .diagnostics
            .extend(native_report.diagnostics.iter().cloned());
        status_report
            .diagnostics
            .extend(native_report.descriptor_diagnostics());
        status_report
            .diagnostics
            .extend(native_report.entry_diagnostics());
        status_report
            .diagnostics
            .extend(feature_report.diagnostics.iter().cloned());

        for package in native_packages {
            let package_diagnostics = native_report.diagnostics_for_plugin(&package.id);
            let load_state = native_load_state(&native_report, &package.id);
            let completed_project_selection = completed_plugins
                .selections
                .iter()
                .find(|selection| selection.id == package.id);
            let native_optional_features = optional_feature_statuses(
                &package,
                completed_project_selection,
                &enabled_plugins,
                &available_capabilities,
                &available_feature_ids,
                &blocked_feature_diagnostics,
                status_target,
            );
            let Some(existing) = status_report
                .plugins
                .iter_mut()
                .find(|plugin| plugin.plugin_id == package.id)
            else {
                status_report.plugins.push(native_plugin_status(
                    &package,
                    manifest,
                    package_diagnostics,
                    load_state,
                    native_optional_features,
                ));
                continue;
            };
            if existing.runtime_crate.is_none() {
                existing.runtime_crate = module_crate(&package, PluginModuleKind::Runtime);
            }
            if existing.editor_crate.is_none() {
                existing.editor_crate = module_crate(&package, PluginModuleKind::Editor);
            }
            if existing.editor_capabilities.is_empty() {
                existing.editor_capabilities = editor_capabilities_for_package(&package);
            }
            if existing.runtime_capabilities.is_empty() {
                existing.runtime_capabilities = runtime_capabilities_for_package(&package);
            }
            if existing.target_modes.is_empty() {
                existing.target_modes = target_modes_for_package(&package);
            }
            if let Some(selection) = manifest
                .plugins
                .selections
                .iter()
                .find(|selection| selection.id == package.id)
            {
                existing.packaging = selection.packaging;
            } else if package
                .default_packaging
                .contains(&ExportPackagingStrategy::NativeDynamic)
            {
                existing.packaging = ExportPackagingStrategy::NativeDynamic;
            }
            existing.package_source = if existing.package_source == "builtin" {
                "builtin + native".to_string()
            } else {
                "native".to_string()
            };
            existing.load_state = load_state;
            merge_optional_feature_statuses(
                &mut existing.optional_features,
                native_optional_features,
            );
            existing.diagnostics.extend(package_diagnostics);
            existing.diagnostics.sort();
            existing.diagnostics.dedup();
        }
        status_report
            .plugins
            .sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));
        status_report
    }
}

fn native_plugin_status(
    package: &PluginPackageManifest,
    manifest: &ProjectManifest,
    mut diagnostics: Vec<String>,
    load_state: String,
    optional_features: Vec<EditorPluginFeatureStatus>,
) -> EditorPluginStatus {
    let project_selection = manifest
        .plugins
        .selections
        .iter()
        .find(|selection| selection.id == package.id);
    diagnostics.push("native plugin discovered outside builtin catalog".to_string());
    diagnostics.sort();
    diagnostics.dedup();
    EditorPluginStatus {
        plugin_id: package.id.clone(),
        display_name: package.display_name.clone(),
        package_source: "native".to_string(),
        load_state,
        enabled: project_selection
            .map(|selection| selection.enabled)
            .unwrap_or(false),
        required: project_selection
            .map(|selection| selection.required)
            .unwrap_or(false),
        target_modes: project_selection
            .map(|selection| selection.target_modes.clone())
            .filter(|modes| !modes.is_empty())
            .unwrap_or_else(|| target_modes_for_package(package)),
        packaging: project_selection
            .map(|selection| selection.packaging)
            .unwrap_or_else(|| default_packaging_for_native_package(package)),
        runtime_crate: project_selection
            .and_then(|selection| selection.runtime_crate.clone())
            .or_else(|| module_crate(package, PluginModuleKind::Runtime)),
        editor_crate: project_selection
            .and_then(|selection| selection.editor_crate.clone())
            .or_else(|| module_crate(package, PluginModuleKind::Editor)),
        runtime_capabilities: runtime_capabilities_for_package(package),
        editor_capabilities: editor_capabilities_for_package(package),
        optional_features,
        diagnostics,
    }
}

fn merge_optional_feature_statuses(
    target: &mut Vec<EditorPluginFeatureStatus>,
    source: Vec<EditorPluginFeatureStatus>,
) {
    for feature in source {
        if let Some(existing) = target.iter_mut().find(|existing| existing.id == feature.id) {
            *existing = feature;
        } else {
            target.push(feature);
        }
    }
    target.sort_by(|left, right| left.id.cmp(&right.id));
}

fn target_modes_for_package(package: &PluginPackageManifest) -> Vec<RuntimeTargetMode> {
    let mut modes = Vec::new();
    for mode in package
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !modes.contains(&mode) {
            modes.push(mode);
        }
    }
    modes
}

fn default_packaging_for_native_package(
    package: &PluginPackageManifest,
) -> ExportPackagingStrategy {
    if package
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic)
    {
        ExportPackagingStrategy::NativeDynamic
    } else {
        package
            .default_packaging
            .first()
            .copied()
            .unwrap_or(ExportPackagingStrategy::NativeDynamic)
    }
}
