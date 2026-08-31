use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::project::{AssetMetaDocument, ProjectManifest, ProjectPaths};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetManager, AssetUri, DataAsset, DataAssetFormat, FunctionAssetImporter, ImportedAsset,
    ProjectAssetManager,
};
use crate::core::framework::project::ProjectPluginManifest;
use crate::core::resource::ResourceKind;
use crate::plugin::{
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};
use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

#[test]
fn runtime_extension_registry_installs_asset_importers_before_project_open() {
    let mut extensions = RuntimeExtensionRegistry::default();
    extensions
        .register_asset_importer(weather_importer())
        .expect("plugin asset importer should register in extension registry");

    let manager = ProjectAssetManager::default();
    extensions
        .apply_asset_importers_to_project_asset_manager(&manager)
        .expect("plugin asset importers should install into the asset manager before open");

    let (root, paths) = write_weather_project("plugin_importer_install");

    AssetManager::open_project(&manager, root.to_string_lossy().as_ref())
        .expect("project should scan with plugin importer installed");

    assert_weather_asset_imported(&manager, &paths);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_module_registration_reports_install_asset_importers_before_project_open() {
    let plugin = WeatherImporterRuntimePlugin::new();
    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);
    let manifest = ProjectPluginManifest {
        selections: vec![registration.project_selection.clone()],
    };
    let report = crate::builtin::runtime_modules_for_target_with_plugin_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        Some(&manifest),
        [&registration],
    )
    .expect("asset importer module composition should compile");

    let runtime = crate::core::CoreRuntime::new();
    for module in report.modules() {
        runtime.register_module(module.descriptor()).unwrap();
    }
    runtime
        .activate_module(crate::asset::ASSET_MODULE_NAME)
        .unwrap();
    let core = runtime.handle();
    let manager = crate::core::manager::resolve_manager_service(
        &core,
        crate::asset::project_asset_manager_handle(&core)
            .expect("project asset manager handle should be registered"),
    )
    .expect("project asset manager should activate with plugin importers");

    let (root, paths) = write_weather_project("runtime_module_importer_install");
    AssetManager::open_project(manager.as_ref(), root.to_string_lossy().as_ref())
        .expect("project should scan with importer from runtime module report");

    assert_weather_asset_imported(manager.as_ref(), &paths);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_plugin_registration_report_validates_shadowed_manifest_asset_importers() {
    let plugin = ShadowedInvalidImporterRuntimePlugin::new();
    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(!registration.is_success());
    assert_eq!(
        registration
            .extensions
            .asset_importers()
            .descriptors()
            .len(),
        1
    );
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("asset importer registration failed")
            && diagnostic.contains("weather.data")
            && diagnostic.contains("source extension or full suffix")
    }));

    let catalog = RuntimePluginCatalog::from_registration_reports([registration], []);

    assert!(!catalog.is_success());
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("asset importer registration failed")
            && diagnostic.contains("weather.data")
            && diagnostic.contains("source extension or full suffix")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_registers_manifest_asset_importers_once() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_asset_importer(
                AssetImporterDescriptor::new(
                    "weather.data",
                    "weather",
                    crate::asset::AssetKind::Data,
                    1,
                )
                .with_source_extensions(["weather"]),
            ),
    );

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    let descriptors = registration.extensions.asset_importers().descriptors();
    assert_eq!(descriptors.len(), 1);
    assert_eq!(descriptors[0].id, "weather.data");
}

#[test]
fn native_runtime_plugin_registration_report_diagnoses_invalid_manifest_asset_importers_once() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_asset_importer(AssetImporterDescriptor::new(
                "weather.data",
                "weather",
                crate::asset::AssetKind::Data,
                1,
            )),
    );

    assert!(!registration.is_success());
    assert!(registration
        .extensions
        .asset_importers()
        .descriptors()
        .is_empty());
    let importer_diagnostics = registration
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.contains("asset importer registration failed")
                && diagnostic.contains("weather.data")
                && diagnostic.contains("source extension or full suffix")
        })
        .count();
    assert_eq!(importer_diagnostics, 1, "{:?}", registration.diagnostics);
}

fn assert_weather_asset_imported(manager: &ProjectAssetManager, paths: &ProjectPaths) {
    let status = AssetManager::asset_status(manager, "res://weather/storm.weather")
        .expect("plugin imported asset status");
    assert!(status.imported);
    assert_eq!(status.kind, ResourceKind::Data);

    let meta = AssetMetaDocument::load(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("weather")
            .join("storm.weather.zmeta"),
    )
    .unwrap();
    assert_eq!(meta.importer_id, "weather.data");
}

fn weather_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new("weather.data", "weather", crate::asset::AssetKind::Data, 1)
            .with_source_extensions(["weather"]),
        import_weather_data,
    )
}

fn write_weather_project(label: &str) -> (PathBuf, ProjectPaths) {
    let root = unique_temp_project_root(label);
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Plugin Importer Install",
        AssetUri::parse("res://weather/storm.weather").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let asset_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("weather")
        .join("storm.weather");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    fs::write(&asset_path, br#"{ "clouds": true }"#).unwrap();
    (root, paths)
}

fn import_weather_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text: String::from_utf8_lossy(&context.source_bytes).into_owned(),
            canonical_json: serde_json::json!({ "kind": "weather" }),
        }),
    ))
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_runtime_{label}_{unique}"))
}

#[derive(Debug)]
struct WeatherImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl WeatherImporterRuntimePlugin {
    fn new() -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                "weather",
                "Weather",
                RuntimePluginId::Particles,
                "zircon_plugin_weather_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.weather")
            .build(),
        }
    }
}

impl RuntimePlugin for WeatherImporterRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_asset_importer(weather_importer())
    }
}

#[derive(Debug)]
struct ShadowedInvalidImporterRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl ShadowedInvalidImporterRuntimePlugin {
    fn new() -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                "weather",
                "Weather",
                RuntimePluginId::Particles,
                "zircon_plugin_weather_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.weather")
            .build(),
        }
    }
}

impl RuntimePlugin for ShadowedInvalidImporterRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor
            .package_manifest()
            .with_asset_importer(AssetImporterDescriptor::new(
                "weather.data",
                "weather",
                crate::asset::AssetKind::Data,
                1,
            ))
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_asset_importer(weather_importer())
    }
}
