use std::collections::BTreeMap;
use std::path::Path;

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime::plugin::native::{NativePluginLoadReport, NativePluginLoader};

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
            NativePluginLoader.load_discovered_editor(self.plugin_directory(project_root));
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
            NativePluginLoader.load_discovered_editor(self.plugin_directory(project_root));
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
    let mut contribution_materialization =
        materialize_native_editor_contributions(native_report, &include_package);
    let native_package_roots = native_report
        .discovered()
        .iter()
        .filter_map(|candidate| {
            candidate
                .manifest_path
                .parent()
                .map(|root| (candidate.plugin_id.clone(), root.to_path_buf()))
        })
        .collect::<BTreeMap<_, _>>();
    let native_projection = native_report.projection();
    native_projection
        .package_manifests()
        .iter()
        .cloned()
        .filter(package_declares_editor_contribution)
        .filter(|package| include_package(&package.id))
        .map(|package| {
            let plugin_id = package.id.clone();
            let native_entry_is_usable = native_projection.is_loaded(&plugin_id)
                && contribution_materialization.is_registration_usable(&plugin_id);
            let (mut extensions, contribution_diagnostics) =
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
            native_editor_registration_from_package(package, extensions, diagnostics)
        })
        .collect()
}

fn native_editor_plugin_is_selected(selections: &ProjectPluginManifest, package_id: &str) -> bool {
    selections.selections.iter().any(|selection| {
        selection.id == package_id
            && selection.enabled
            && selection.supports_target(RuntimeTargetMode::EditorHost)
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::{
        ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
    };
    use zircon_runtime::plugin::native::{NativePluginCandidate, NativePluginLoadReport};
    use zircon_runtime::plugin::{PluginModuleManifest, PluginPackageManifest};

    use super::{
        native_editor_plugin_is_selected, native_editor_registration_reports_from_load_report,
    };

    fn selection(
        id: &str,
        enabled: bool,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> ProjectPluginSelection {
        ProjectPluginSelection {
            id: id.to_string(),
            enabled,
            required: false,
            target_modes: target_modes.into_iter().collect(),
            packaging: ExportPackagingStrategy::NativeDynamic,
            runtime_crate: None,
            editor_crate: None,
            features: Vec::new(),
        }
    }

    #[test]
    fn native_editor_registration_selection_requires_enabled_editor_host_support() {
        let selections = ProjectPluginManifest {
            selections: vec![
                selection("disabled", false, [RuntimeTargetMode::EditorHost]),
                selection("client_only", true, [RuntimeTargetMode::ClientRuntime]),
                selection("unbounded", true, []),
                selection("editor", true, [RuntimeTargetMode::EditorHost]),
            ],
        };

        assert!(!native_editor_plugin_is_selected(&selections, "missing"));
        assert!(!native_editor_plugin_is_selected(&selections, "disabled"));
        assert!(!native_editor_plugin_is_selected(
            &selections,
            "client_only"
        ));
        assert!(native_editor_plugin_is_selected(&selections, "unbounded"));
        assert!(native_editor_plugin_is_selected(&selections, "editor"));
    }

    #[test]
    fn selected_native_load_failure_remains_visible_to_the_plugin_manager() {
        let package_id = "fixture.native-editor";
        let report = NativePluginLoadReport {
            discovered: vec![NativePluginCandidate {
                plugin_id: package_id.to_string(),
                package_manifest: PluginPackageManifest::new(package_id, "Fixture editor")
                    .with_module(PluginModuleManifest::editor(
                        "fixture.native-editor.editor",
                        "fixture_native_editor",
                    )),
                manifest_path: PathBuf::from("fixture.native-editor/plugin.toml"),
                library_path: PathBuf::from("fixture.native-editor/native/plugin.dll"),
            }],
            diagnostics: vec![
                "native plugin fixture.native-editor: editor entry could not load".to_string(),
            ],
            ..NativePluginLoadReport::default()
        };

        let registrations =
            native_editor_registration_reports_from_load_report(&report, |_| true, true);

        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].package_manifest.id, package_id);
        assert!(!registrations[0].is_success());
        assert!(registrations[0]
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("editor entry could not load")));
    }
}
