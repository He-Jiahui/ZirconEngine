use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
};
use zircon_runtime::plugin::native::discovery::load_discovered_native_editor_plugins;

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
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains(package_id)
            && diagnostic.contains("library-open")
            && diagnostic.contains("artifact missing")));
}

#[test]
fn selected_test_fixture_is_not_projected_as_a_product_editor_plugin() {
    let package_id = "fixture.native-editor";
    let fixture = TempNativePluginRoot::new("test-carrier-exclusion");
    fixture.write_editor_manifest(package_id);
    let manifest_path = fixture.path().join(package_id).join("plugin.toml");
    let manifest = fs::read_to_string(&manifest_path).expect("read fixture manifest");
    fs::write(
        &manifest_path,
        format!("package_role = \"test_fixture\"\n{manifest}"),
    )
    .expect("mark package as test-only carrier");
    let native_report = load_discovered_native_editor_plugins(fixture.path());

    let registrations =
        native_editor_registration_reports_from_load_report(&native_report, |_| true, true);
    assert!(
        registrations.is_empty(),
        "test carriers must not enter the editor product catalog"
    );
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
