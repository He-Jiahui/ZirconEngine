use std::collections::BTreeMap;
use std::path::Path;

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginManifest;
use zircon_runtime::plugin::native::{
    NativePluginLoadReport, load_discovered_native_editor_plugins,
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
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::{
        ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
    };
    use zircon_runtime::plugin::native::load_discovered_native_editor_plugins;

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
        let fixture = TempNativePluginRoot::new("selected-load-failure");
        fixture.write_editor_manifest(package_id);
        let report = load_discovered_native_editor_plugins(fixture.path());

        let registrations =
            native_editor_registration_reports_from_load_report(&report, |_| true, true);

        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].package_manifest.id, package_id);
        assert!(!registrations[0].is_success());
        assert!(registrations[0].diagnostics.iter().any(
            |diagnostic| diagnostic.contains(package_id)
                && diagnostic.contains("library-open")
                && diagnostic.contains("artifact missing")
        ));
    }

    struct TempNativePluginRoot {
        path: PathBuf,
    }

    impl TempNativePluginRoot {
        fn new(label: &str) -> Self {
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "zircon-editor-native-registration-{label}-{}-{stamp}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create native registration fixture root");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn write_editor_manifest(&self, package_id: &str) {
            let package_root = self.path.join(package_id);
            fs::create_dir_all(&package_root).expect("create native registration package root");
            let manifest = format!(
                r#"id = "{package_id}"
version = "0.1.0"
display_name = "Fixture editor"

[[modules]]
name = "{package_id}.editor"
kind = "editor"
crate_name = "fixture_native_editor"
"#
            );
            fs::write(package_root.join("plugin.toml"), manifest)
                .expect("write native registration manifest");
        }
    }

    impl Drop for TempNativePluginRoot {
        fn drop(&mut self) {
            debug_assert!(self.path.starts_with(std::env::temp_dir()));
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
