use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ExportProfile, ExportTargetPlatform, ProjectPluginFeatureSelection,
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use zircon_runtime::core::manager::ManagerResolver;
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::plugin::{
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest, PluginPackageRole,
    RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginAvailabilityCategory,
    RuntimePluginDescriptor, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use super::super::{
    bootstrap_export_runtime, bootstrap_export_runtime_with_native_plugins_from_export_root,
    ExportRuntimeBootstrapConfig, ExportRuntimePluginFeatureRegistrationProvider,
    ProductComposition, ProductConfigSource, ProductConfigSourceSet, ProductHostConfigError,
    ProductRoleRequest,
};

#[test]
fn export_feature_provider_stamps_admitted_role_over_self_declared_default() {
    let profile = ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Client2d,
    );
    let manifest = ProjectPluginManifest::default();
    let unstamped = ExportRuntimeBootstrapConfig::new(manifest.clone(), profile.clone())
        .with_runtime_plugin_feature_registration_providers([
            ExportRuntimePluginFeatureRegistrationProvider::new(default_production_feature_report),
        ]);
    assert_eq!(
        unstamped.runtime_plugin_feature_registrations[0].provider_package_role,
        PluginPackageRole::TestFixture,
        "a bare linked provider must fail closed even when its function defaults to Production"
    );

    for role in [
        PluginPackageRole::Production,
        PluginPackageRole::Sample,
        PluginPackageRole::TestFixture,
    ] {
        let config = ExportRuntimeBootstrapConfig::new(manifest.clone(), profile.clone())
            .with_runtime_plugin_feature_registration_providers([
                ExportRuntimePluginFeatureRegistrationProvider::new(
                    default_production_feature_report,
                )
                .with_provider_package_id("independent_carrier")
                .with_admitted_source_identity(
                    "sound.exported",
                    "sound",
                    "independent_carrier",
                    "zircon_plugin_sound_exported_runtime",
                    role,
                ),
            ]);
        let report = &config.runtime_plugin_feature_registrations[0];
        assert_eq!(
            report.provider_package_id.as_deref(),
            Some("independent_carrier")
        );
        assert_eq!(report.provider_package_role, role);
    }
}

#[test]
fn exported_required_feature_rejects_sample_and_fixture_carrier_roles() {
    for role in [PluginPackageRole::Sample, PluginPackageRole::TestFixture] {
        let mut config = export_bootstrap_config(
            [RuntimePluginId::Sound],
            [linked_sound_registration_report()],
        );
        config.project_plugins.selections[0].features.push(
            ProjectPluginFeatureSelection::new("sound.exported")
                .enabled(true)
                .required(true)
                .with_provider_package_id("independent_carrier"),
        );
        config
            .project_plugins
            .selections
            .push(ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::new("independent_carrier"),
                true,
                false,
            ));
        let config = config.with_runtime_plugin_feature_registration_providers([
            ExportRuntimePluginFeatureRegistrationProvider::new(default_production_feature_report)
                .with_provider_package_id("independent_carrier")
                .with_admitted_source_identity(
                    "sound.exported",
                    "sound",
                    "independent_carrier",
                    "zircon_plugin_sound_exported_runtime",
                    role,
                ),
        ]);

        let error = bootstrap_export_runtime(config)
            .expect_err("test-only linked feature must not satisfy required product selection");
        assert!(error.to_string().contains("sound.exported"), "{error}");
    }
}

#[test]
fn exported_required_feature_accepts_an_admitted_production_carrier() {
    let config_file = IsolatedConfigFile::new();
    let mut config = export_bootstrap_config(
        [RuntimePluginId::Sound],
        [linked_sound_registration_report()],
    );
    config.project_plugins.selections[0].features.push(
        ProjectPluginFeatureSelection::new("sound.exported")
            .enabled(true)
            .required(true)
            .with_provider_package_id("independent_carrier"),
    );
    config
        .project_plugins
        .selections
        .push(ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::new("independent_carrier"),
            true,
            false,
        ));
    let config = config
        .with_config_file_path(config_file.path())
        .with_runtime_plugin_feature_registration_providers([
            ExportRuntimePluginFeatureRegistrationProvider::new(default_production_feature_report)
                .with_provider_package_id("independent_carrier")
                .with_admitted_source_identity(
                    "sound.exported",
                    "sound",
                    "independent_carrier",
                    "zircon_plugin_sound_exported_runtime",
                    PluginPackageRole::Production,
                ),
        ]);

    let composition = bootstrap_export_runtime(config)
        .expect("admitted production feature should satisfy required export selection");
    {
        assert_export_config_file_loaded(&composition);
        assert!(composition
            .module_selection_report()
            .module_keys()
            .contains(&"sound.exported.runtime"));
    }
    super::product_composition::close_owner::close_composition(composition);
}

#[test]
fn admitted_role_cannot_promote_a_different_returned_feature() {
    let mut config = export_bootstrap_config(
        [RuntimePluginId::Sound],
        [linked_sound_registration_report()],
    );
    config.project_plugins.selections[0].features.push(
        ProjectPluginFeatureSelection::new("sound.unadmitted")
            .enabled(true)
            .required(true)
            .with_provider_package_id("independent_carrier"),
    );
    config
        .project_plugins
        .selections
        .push(ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::new("independent_carrier"),
            true,
            false,
        ));
    let config = config.with_runtime_plugin_feature_registration_providers([
        ExportRuntimePluginFeatureRegistrationProvider::new(wrong_feature_report)
            .with_provider_package_id("independent_carrier")
            .with_admitted_source_identity(
                "sound.exported",
                "sound",
                "independent_carrier",
                "zircon_plugin_sound_exported_runtime",
                PluginPackageRole::Production,
            ),
    ]);
    let report = &config.runtime_plugin_feature_registrations[0];
    assert_eq!(report.provider_package_role, PluginPackageRole::TestFixture);
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("admitted source identity")));
    let error = bootstrap_export_runtime(config)
        .expect_err("an unadmitted returned feature must not satisfy a required selection");
    assert!(error.to_string().contains("sound.unadmitted"), "{error}");
}

#[test]
fn admitted_role_rejects_wrong_owner_or_runtime_crate() {
    for report_fn in [
        wrong_owner_report as fn() -> RuntimePluginFeatureRegistrationReport,
        wrong_runtime_crate_report,
    ] {
        let config = ExportRuntimeBootstrapConfig::new(
            ProjectPluginManifest::default(),
            ExportProfile::new(
                "client",
                RuntimeTargetMode::ClientRuntime,
                ExportTargetPlatform::Windows,
                RuntimeProfileId::Client2d,
            ),
        )
        .with_runtime_plugin_feature_registration_providers([
            ExportRuntimePluginFeatureRegistrationProvider::new(report_fn)
                .with_admitted_source_identity(
                    "sound.exported",
                    "sound",
                    "sound",
                    "zircon_plugin_sound_exported_runtime",
                    PluginPackageRole::Production,
                ),
        ]);
        let report = &config.runtime_plugin_feature_registrations[0];
        assert_eq!(report.provider_package_role, PluginPackageRole::TestFixture);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("admitted source identity")));
    }
}

fn wrong_feature_report() -> RuntimePluginFeatureRegistrationReport {
    let mut report = default_production_feature_report();
    report.manifest.id = "sound.unadmitted".to_owned();
    report
}

fn wrong_owner_report() -> RuntimePluginFeatureRegistrationReport {
    let mut report = default_production_feature_report();
    report.manifest.owner_plugin_id = "animation".to_owned();
    report
}

fn wrong_runtime_crate_report() -> RuntimePluginFeatureRegistrationReport {
    let mut report = default_production_feature_report();
    report.manifest.modules[0].crate_name = "zircon_plugin_other_runtime".to_owned();
    report
}

#[test]
fn admitted_owner_embedded_feature_rejects_a_self_declared_external_provider() {
    let config = ExportRuntimeBootstrapConfig::new(
        ProjectPluginManifest::default(),
        ExportProfile::new(
            "client",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Client2d,
        ),
    )
    .with_runtime_plugin_feature_registration_providers([
        ExportRuntimePluginFeatureRegistrationProvider::new(wrong_provider_report)
            .with_admitted_source_identity(
                "sound.exported",
                "sound",
                "sound",
                "zircon_plugin_sound_exported_runtime",
                PluginPackageRole::Production,
            ),
    ]);
    let report = &config.runtime_plugin_feature_registrations[0];
    assert_eq!(report.provider_package_role, PluginPackageRole::TestFixture);
    assert_eq!(report.provider_package_id.as_deref(), Some("sound"));
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("admitted source identity")));
}

fn wrong_provider_report() -> RuntimePluginFeatureRegistrationReport {
    let mut report = default_production_feature_report();
    report.provider_package_id = Some("rogue_carrier".to_owned());
    report.manifest.provider_package_id = Some("rogue_carrier".to_owned());
    report.project_selection.provider_package_id = Some("rogue_carrier".to_owned());
    report
}

fn default_production_feature_report() -> RuntimePluginFeatureRegistrationReport {
    let mut extensions = RuntimeExtensionRegistry::default();
    extensions
        .register_module(ModuleDescriptor::new(
            "sound.exported.runtime",
            "Linked sound feature module",
        ))
        .expect("linked feature module fixture");
    RuntimePluginFeatureRegistrationReport {
        manifest: PluginFeatureBundleManifest::new("sound.exported", "Exported", "sound")
            .with_dependency(PluginFeatureDependency::primary(
                "sound",
                "runtime.plugin.sound",
            ))
            .with_runtime_module(PluginModuleManifest::runtime(
                "sound.exported.runtime",
                "zircon_plugin_sound_exported_runtime",
            )),
        provider_package_id: None,
        provider_package_role: PluginPackageRole::Production,
        project_selection: ProjectPluginFeatureSelection::new("sound.exported"),
        extensions,
        diagnostics: Vec::new(),
    }
}

#[test]
fn export_runtime_bootstrap_uses_linked_registration_reports() {
    let config_file = IsolatedConfigFile::new();
    let bootstrap = bootstrap_export_runtime(
        export_bootstrap_config(
            [RuntimePluginId::Sound],
            [linked_sound_registration_report()],
        )
        .with_config_file_path(config_file.path()),
    )
    .expect("export bootstrap facade should use linked runtime registrations");
    {
        assert_export_config_file_loaded(&bootstrap);

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
    super::product_composition::close_owner::close_composition(bootstrap);
}

#[test]
fn native_export_runtime_bootstrap_merges_linked_and_native_reports() {
    let config_file = IsolatedConfigFile::new();
    let export_root = unique_export_root("zircon_app_export_bootstrap_merge");
    write_virtual_geometry_native_package(&export_root);
    // The missing native library still contributes diagnostics, but no live product module.
    let mut config = export_bootstrap_config(
        [RuntimePluginId::Sound, RuntimePluginId::VirtualGeometry],
        [linked_sound_registration_report()],
    );
    let virtual_geometry = config
        .project_plugins
        .selections
        .iter_mut()
        .find(|selection| selection.id == RuntimePluginId::VirtualGeometry.key())
        .expect("export fixture must select VirtualGeometry");
    virtual_geometry.required = false;
    virtual_geometry.packaging = ExportPackagingStrategy::NativeDynamic;
    let config = config.with_config_file_path(config_file.path());
    let bootstrap =
        bootstrap_export_runtime_with_native_plugins_from_export_root(config, &export_root)
            .expect("missing optional native DLL must not discard the linked runtime registration");
    {
        assert_export_config_file_loaded(&bootstrap);

        let module_keys = bootstrap.module_selection_report().module_keys();
        assert!(module_keys.contains(&"LinkedSoundPlugin"));
        assert!(!module_keys.contains(&"virtual_geometry.runtime"));
        assert!(bootstrap
            .module_selection_report()
            .runtime_plugin_availability
            .contains(
                RuntimePluginAvailabilityCategory::Linked,
                RuntimePluginId::Sound
            ));
        assert!(!bootstrap
            .module_selection_report()
            .runtime_plugin_availability
            .contains(
                RuntimePluginAvailabilityCategory::NativeDynamic,
                RuntimePluginId::VirtualGeometry
            ));
        assert!(bootstrap
            .native_plugin_host()
            .expect("native export composition must retain its host")
            .loaded_plugin_ids(zircon_runtime::plugin::PluginModuleKind::Runtime)
            .expect("live host must expose Runtime plugin ids")
            .is_empty());
        assert!(bootstrap.diagnostics().iter().any(|diagnostic| {
            diagnostic.contains("native plugin virtual_geometry skipped because library is missing")
        }));
    }
    super::product_composition::close_owner::close_composition(bootstrap);
    fs::remove_dir_all(export_root).expect("remove only the isolated export fixture");
}

#[test]
fn required_native_export_plugin_rejects_discovery_without_a_library() {
    let export_root = unique_export_root("zircon_app_export_required_native_missing_library");
    write_virtual_geometry_native_package(&export_root);
    let mut config = export_bootstrap_config(
        [RuntimePluginId::Sound, RuntimePluginId::VirtualGeometry],
        [linked_sound_registration_report()],
    );
    let virtual_geometry = config
        .project_plugins
        .selections
        .iter_mut()
        .find(|selection| selection.id == RuntimePluginId::VirtualGeometry.key())
        .expect("export fixture must select VirtualGeometry");
    virtual_geometry.packaging = ExportPackagingStrategy::NativeDynamic;

    let error = bootstrap_export_runtime_with_native_plugins_from_export_root(config, &export_root)
        .expect_err("discovery without a loaded DLL cannot admit required NativeDynamic");
    assert!(
        error
            .to_string()
            .contains("required native plugin virtual_geometry was not admitted"),
        "{error}"
    );

    assert!(!export_root.join("plugins/virtual_geometry/native").exists());
    fs::remove_dir_all(export_root).expect("remove only the isolated export fixture");
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

struct IsolatedConfigFile {
    path: PathBuf,
}

impl IsolatedConfigFile {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "zircon-app-export-config-{}-{stamp}-{id}.json",
            std::process::id()
        ));
        fs::write(&path, br#"{"fixture.export_bootstrap":true}"#).unwrap();
        Self { path }
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Drop for IsolatedConfigFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn assert_export_config_file_loaded(composition: &ProductComposition) {
    let resolver = ManagerResolver::new(composition.core().clone());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    assert_eq!(
        config.get_value("fixture.export_bootstrap"),
        Some(serde_json::json!(true))
    );
}
