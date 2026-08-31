use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ExportProfile, ExportTargetPlatform, ProjectPluginManifest,
    ProjectPluginSelection, RuntimeProfileId,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginAvailabilityCategory,
    RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use super::super::{
    bootstrap_export_runtime, bootstrap_export_runtime_with_native_plugins_from_export_root,
    ExportRuntimeBootstrapConfig, ProductConfigSource, ProductConfigSourceSet,
    ProductHostConfigError, ProductRoleRequest,
};

#[test]
fn export_runtime_bootstrap_uses_linked_registration_reports() {
    let bootstrap = bootstrap_export_runtime(export_bootstrap_config(
        [RuntimePluginId::Sound],
        [linked_sound_registration_report()],
    ))
    .expect("export bootstrap facade should use linked runtime registrations");

    assert!(bootstrap
        .module_selection_report()
        .module_keys()
        .contains(&"LinkedSoundPlugin"));
    assert_eq!(
        bootstrap.module_selection_report().runtime_profile,
        Some(RuntimeProfileId::Client2d),
        "export runtime composition must preserve the profile encoded by the export receipt"
    );
    assert_eq!(
        bootstrap.module_selection_report().product_role,
        ProductRoleRequest::DesktopClient
    );
    assert_eq!(
        bootstrap
            .module_selection_report()
            .product_config_provenance
            .runtime_profile(),
        ProductConfigSource::ExportProfile
    );
    assert_eq!(
        bootstrap
            .module_selection_report()
            .product_config_provenance
            .project_plugins(),
        ProductConfigSourceSet::single(ProductConfigSource::RuntimeProfile)
            .with(ProductConfigSource::ExportProfile)
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
    assert!(bootstrap
        .module_selection_report()
        .runtime_plugin_availability
        .contains(
            RuntimePluginAvailabilityCategory::Linked,
            RuntimePluginId::Sound
        ));
    assert!(bootstrap
        .module_selection_report()
        .runtime_plugin_availability
        .contains(
            RuntimePluginAvailabilityCategory::NativeDynamic,
            RuntimePluginId::VirtualGeometry
        ));
    assert!(bootstrap.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("native plugin virtual_geometry skipped because library is missing")
    }));

    let _ = fs::remove_dir_all(export_root);
}

#[test]
fn invalid_export_product_config_fails_before_native_root_access() {
    let mut config = export_bootstrap_config([], []);
    config.export_profile.runtime_profile_id = Some(RuntimeProfileId::Server);
    let missing_export_root = unique_export_root("unreachable_native_root");

    let error =
        bootstrap_export_runtime_with_native_plugins_from_export_root(config, &missing_export_root)
            .unwrap_err();

    assert!(error.to_string().contains("zircon_app product host config"));
    assert!(!missing_export_root.exists());
}

#[test]
fn export_product_role_is_derived_from_the_export_profile() {
    let cases = [
        (
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            ProductRoleRequest::DesktopClient,
        ),
        (
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Android,
            ProductRoleRequest::AndroidClient,
        ),
        (
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::WebGpu,
            ProductRoleRequest::WebClient,
        ),
        (
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Ios,
            ProductRoleRequest::Embedded,
        ),
        (
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Android,
            ProductRoleRequest::AndroidClient,
        ),
        (
            RuntimeTargetMode::EditorHost,
            ExportTargetPlatform::WebGpu,
            ProductRoleRequest::WebClient,
        ),
        (
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Headless,
            ProductRoleRequest::Embedded,
        ),
        (
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Headless,
            ProductRoleRequest::Server,
        ),
    ];

    for (target_mode, target_platform, expected_role) in cases {
        let config = ExportRuntimeBootstrapConfig::new(
            ProjectPluginManifest::default(),
            ExportProfile::new(
                "role-projection",
                target_mode,
                target_platform,
                match target_mode {
                    RuntimeTargetMode::ServerRuntime => RuntimeProfileId::Server,
                    RuntimeTargetMode::EditorHost => RuntimeProfileId::Editor,
                    RuntimeTargetMode::ClientRuntime => RuntimeProfileId::Client2d,
                },
            ),
        );

        assert_eq!(config.entry_config().role_request(), expected_role);
    }
}

#[test]
fn unowned_export_hosts_fail_closed_with_their_product_role() {
    let cases = [
        (
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Android,
            ProductRoleRequest::AndroidClient,
        ),
        (
            RuntimeTargetMode::EditorHost,
            ExportTargetPlatform::WebGpu,
            ProductRoleRequest::WebClient,
        ),
        (
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Ios,
            ProductRoleRequest::Embedded,
        ),
    ];

    for (target_mode, target_platform, expected_role) in cases {
        let error = ExportRuntimeBootstrapConfig::new(
            ProjectPluginManifest::default(),
            ExportProfile::new(
                "unsupported-host",
                target_mode,
                target_platform,
                match target_mode {
                    RuntimeTargetMode::ServerRuntime => RuntimeProfileId::Server,
                    RuntimeTargetMode::EditorHost => RuntimeProfileId::Editor,
                    RuntimeTargetMode::ClientRuntime => RuntimeProfileId::Client2d,
                },
            ),
        )
        .entry_config()
        .resolve()
        .unwrap_err();

        assert_eq!(
            error,
            ProductHostConfigError::UnsupportedProductRole(expected_role)
        );
    }
}

fn export_bootstrap_config<const REQUIRED: usize, const LINKED: usize>(
    required_plugins: [RuntimePluginId; REQUIRED],
    linked_reports: [RuntimePluginRegistrationReport; LINKED],
) -> ExportRuntimeBootstrapConfig {
    ExportRuntimeBootstrapConfig::new(
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
