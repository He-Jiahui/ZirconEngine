use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::project::{AssetMetaDocument, ProjectManifest, ProjectPaths};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetManager, AssetUri, DataAsset, DataAssetFormat, FunctionAssetImporter, ImportedAsset,
    ProjectAssetManager,
};
use crate::core::resource::ResourceKind;
use crate::plugin::{
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};
use crate::{RuntimePluginId, RuntimeTargetMode};

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
    let report = crate::runtime_modules_for_target_with_plugin_registration_reports(
        RuntimeTargetMode::ClientRuntime,
        None,
        [&registration],
    );
    assert!(report.errors.is_empty(), "{:?}", report.errors);

    let runtime = crate::core::CoreRuntime::new();
    for module in report.modules {
        runtime.register_module(module.descriptor()).unwrap();
    }
    runtime
        .activate_module(crate::asset::ASSET_MODULE_NAME)
        .unwrap();
    let manager = runtime
        .resolve_manager::<ProjectAssetManager>(crate::asset::PROJECT_ASSET_MANAGER_NAME)
        .expect("project asset manager should activate with plugin importers");

    let (root, paths) = write_weather_project("runtime_module_importer_install");
    AssetManager::open_project(manager.as_ref(), root.to_string_lossy().as_ref())
        .expect("project should scan with importer from runtime module report");

    assert_weather_asset_imported(manager.as_ref(), &paths);

    let _ = fs::remove_dir_all(root);
}

fn assert_weather_asset_imported(manager: &ProjectAssetManager, paths: &ProjectPaths) {
    let status = AssetManager::asset_status(manager, "res://weather/storm.weather")
        .expect("plugin imported asset status");
    assert!(status.imported);
    assert_eq!(status.kind, ResourceKind::Data);

    let meta = AssetMetaDocument::load(
        paths
            .assets_root()
            .join("weather")
            .join("storm.weather.meta.toml"),
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
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Plugin Importer Install",
        AssetUri::parse("res://weather/storm.weather").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let asset_path = paths.assets_root().join("weather").join("storm.weather");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    fs::write(&asset_path, br#"{ "clouds": true }"#).unwrap();
    (root, paths)
}

fn import_weather_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    Ok(AssetImportOutcome::new(ImportedAsset::Data(DataAsset {
        uri: context.uri.clone(),
        format: DataAssetFormat::Json,
        text: String::from_utf8_lossy(&context.source_bytes).into_owned(),
        canonical_json: serde_json::json!({ "kind": "weather" }),
    })))
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
            descriptor: RuntimePluginDescriptor::new(
                "weather",
                "Weather",
                RuntimePluginId::Particles,
                "zircon_plugin_weather_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime]),
        }
    }
}

impl RuntimePlugin for WeatherImporterRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_asset_importer(weather_importer())
    }
}
