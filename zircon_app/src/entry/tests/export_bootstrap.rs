use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ExportProfile, ExportTargetPlatform, ProjectPluginManifest,
    ProjectPluginSelection, RuntimeProfileId,
};
use zircon_runtime::plugin::{
    RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginAvailabilityCategory,
    RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use super::super::{
    EntryProfile, ExportRuntimeBootstrapConfig,
    bootstrap_export_runtime_with_native_plugins_from_export_root,
    bootstrap_export_runtime_with_report,
};

#[test]
fn export_runtime_bootstrap_uses_linked_registration_reports() {
    let bootstrap = bootstrap_export_runtime_with_report(export_bootstrap_config(
        [RuntimePluginId::Sound],
        [linked_sound_registration_report()],
    ))
    .expect("export bootstrap facade should use linked runtime registrations");

    assert!(
        bootstrap
            .module_selection_report()
            .module_keys()
            .contains(&"LinkedSoundPlugin")
    );
}

#[test]
fn native_export_runtime_bootstrap_merges_linked_and_native_reports() {
    let export_root = unique_export_root("zircon_app_export_bootstrap_merge");
    write_virtual_geometry_native_package(&export_root);
    let bootstrap = bootstrap_export_runtime_with_native_plugins_from_export_root(
        export_bootstrap_config(
            [RuntimePluginId::Sound, RuntimePluginId::VirtualGeometry],
            [linked_sound_registration_report()],
        ),
        &export_root,
    )
    .expect("export bootstrap facade should merge linked and native runtime registrations");

    let module_keys = bootstrap.module_selection_report().module_keys();
    assert!(module_keys.contains(&"LinkedSoundPlugin"));
    assert!(module_keys.contains(&"virtual_geometry.runtime"));
    assert!(
        bootstrap
            .module_selection_report()
            .runtime_plugin_availability
            .contains(
                RuntimePluginAvailabilityCategory::Linked,
                RuntimePluginId::Sound
            )
    );
    assert!(
        bootstrap
            .module_selection_report()
            .runtime_plugin_availability
            .contains(
                RuntimePluginAvailabilityCategory::NativeDynamic,
                RuntimePluginId::VirtualGeometry
            )
    );
    assert!(bootstrap.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("native plugin virtual_geometry skipped because library is missing")
    }));

    let _ = fs::remove_dir_all(export_root);
}

fn export_bootstrap_config<const REQUIRED: usize, const LINKED: usize>(
    required_plugins: [RuntimePluginId; REQUIRED],
    linked_reports: [RuntimePluginRegistrationReport; LINKED],
) -> ExportRuntimeBootstrapConfig {
    ExportRuntimeBootstrapConfig::new(
        EntryProfile::Runtime,
        RuntimeTargetMode::ClientRuntime,
        ProjectPluginManifest {
            selections: required_plugins
                .into_iter()
                .map(|plugin_id| ProjectPluginSelection::runtime_plugin(plugin_id, true, true))
                .collect(),
        },
        ExportProfile::new(
            "client",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Client2d,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate),
    )
    .with_runtime_plugin_registrations(linked_reports)
}

fn linked_sound_registration_report() -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_plugin(&LinkedSoundPlugin)
}

#[derive(Debug)]
struct LinkedSoundPlugin;

impl RuntimePlugin for LinkedSoundPlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        static DESCRIPTOR: std::sync::OnceLock<RuntimePluginDescriptor> =
            std::sync::OnceLock::new();
        DESCRIPTOR.get_or_init(|| {
            RuntimePluginDescriptor::builder(
                "sound",
                "Sound",
                RuntimePluginId::Sound,
                "zircon_plugin_sound_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.sound")
            .build()
        })
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(ModuleDescriptor::new(
            "LinkedSoundPlugin",
            "Linked sound plugin module",
        ))
    }
}

fn write_virtual_geometry_native_package(export_root: &Path) {
    fs::create_dir_all(export_root.join("plugins/virtual_geometry")).unwrap();
    fs::write(
        export_root.join("plugins/native_plugins.toml"),
        r#"
[[plugins]]
id = "virtual_geometry"
path = "plugins/virtual_geometry"
manifest = "plugins/virtual_geometry/plugin.toml"
"#,
    )
    .unwrap();
    fs::write(
        export_root.join("plugins/virtual_geometry/plugin.toml"),
        r#"
id = "virtual_geometry"
version = "0.1.0"
display_name = "Virtual Geometry"

[[modules]]
name = "virtual_geometry.runtime"
kind = "runtime"
crate_name = "zircon_plugin_virtual_geometry_runtime"
target_modes = ["client_runtime"]
"#,
    )
    .unwrap();
}

fn unique_export_root(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{stamp}"))
}
