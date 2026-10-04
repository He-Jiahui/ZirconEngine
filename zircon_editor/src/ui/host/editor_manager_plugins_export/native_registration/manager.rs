use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ProjectPluginManifest};
use zircon_runtime::plugin::native::{
    discovery::load_discovered_native_editor_plugins, NativePluginLoadReport,
};

use crate::core::plugin::EditorPluginRegistrationReport;

use super::super::super::editor_manager::EditorManager;
use super::native_contribution::materialize_native_editor_contributions;
use super::registration_projection::{
    native_editor_registration_from_package, package_declares_editor_contribution,
};

impl EditorManager {
    pub fn native_editor_plugin_registration_reports(
        &self,
        project_root: impl AsRef<Path>,
    ) -> Vec<EditorPluginRegistrationReport> {
        let native_report =
            load_discovered_native_editor_plugins(self.plugin_directory(project_root));
        native_editor_registration_reports_from_load_report(&native_report, |_| true, false)
    }

    /// Materializes only native editor packages selected for this editor-host project generation.
    ///
    /// The caller supplies the already-open project's selection manifest. Loading, selection,
    /// contribution materialization, and registration projection consume one native load report.
    pub fn selected_native_editor_plugin_registration_reports(
        &self,
        project_root: impl AsRef<Path>,
        selections: &ProjectPluginManifest,
    ) -> Vec<EditorPluginRegistrationReport> {
        let native_report =
            load_discovered_native_editor_plugins(self.plugin_directory(project_root));
        native_editor_registration_reports_from_load_report(
            &native_report,
            |package_id| native_editor_plugin_is_selected(selections, package_id),
            true,
        )
    }

    pub(in crate::ui::host) fn selected_native_editor_plugin_registration_reports_from_load_report(
        &self,
        native_report: &NativePluginLoadReport,
        selections: &ProjectPluginManifest,
    ) -> Vec<EditorPluginRegistrationReport> {
        native_editor_registration_reports_from_load_report(
            native_report,
            |package_id| native_editor_plugin_is_selected(selections, package_id),
            true,
        )
    }
}

fn native_editor_registration_reports_from_load_report(
    native_report: &NativePluginLoadReport,
    include_package: impl Fn(&str) -> bool,
    report_unusable_native_entry: bool,
) -> Vec<EditorPluginRegistrationReport> {
    let native_projection = native_report.projection();
    let product_packages = native_projection
        .package_manifests()
        .iter()
        .filter(|package| package.package_role.is_product_catalog_eligible())
        .map(|package| package.id.as_str())
        .collect::<HashSet<_>>();
    let include_product =
        |package_id: &str| product_packages.contains(package_id) && include_package(package_id);
    let mut contribution_materialization =
        materialize_native_editor_contributions(native_report, &include_product);
    let native_package_roots = native_report
        .discovered()
        .iter()
        .filter(|candidate| {
            candidate
                .package_manifest
                .package_role
                .is_product_catalog_eligible()
        })
        .filter_map(|candidate| {
            candidate
                .manifest_path
                .parent()
                .map(|root| (candidate.plugin_id.clone(), root.to_path_buf()))
        })
        .collect::<BTreeMap<_, _>>();
    native_projection
        .package_manifests()
        .iter()
        .cloned()
        .filter(|package| package.package_role.is_product_catalog_eligible())
        .filter(package_declares_editor_contribution)
        .filter(|package| include_product(&package.id))
        .map(|package| {
            let plugin_id = package.id.clone();
            let native_entry_is_usable = native_projection.is_loaded(&plugin_id)
                && contribution_materialization.is_registration_usable(&plugin_id);
            let (mut extensions, native_command_bindings, contribution_diagnostics) =
                contribution_materialization.take_registration(&plugin_id);
            if let Some(root) = native_package_roots.get(&plugin_id) {
                extensions.bind_ui_template_root(root);
            }
            let mut diagnostics = native_projection.editor_diagnostics_for_plugin(&plugin_id);
            diagnostics.extend(contribution_diagnostics);
            if report_unusable_native_entry && !native_entry_is_usable {
                // Keep selected native packages in the catalog so the manager publishes the
                // existing diagnostics-to-Faulted state instead of silently hiding the failure.
                diagnostics.push(format!(
                    "native editor entry is unavailable for selected plugin `{plugin_id}`"
                ));
            }
            native_editor_registration_from_package(
                package,
                extensions,
                native_command_bindings,
                diagnostics,
            )
        })
        .collect()
}

fn native_editor_plugin_is_selected(selections: &ProjectPluginManifest, package_id: &str) -> bool {
    selections.selections.iter().any(|selection| {
        selection.id == package_id
            && selection.enabled
            && selection.packaging == ExportPackagingStrategy::NativeDynamic
            && selection.supports_target(RuntimeTargetMode::EditorHost)
    })
}

#[cfg(test)]
#[path = "tests/manager.rs"]
mod tests;
