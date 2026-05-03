use serde_json::{Map, Value};
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    DataAsset, DataAssetFormat, FunctionAssetImporter, ImportedAsset,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};

pub const PLUGIN_ID: &str = "asset_importer.data";
pub const IMPORTER_FAMILY: &str = "data";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_asset_importer_data_runtime";
pub const MODULE_NAME: &str = "DataImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.asset_importer.data";
pub const TOML_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.data.toml";
pub const JSON_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.data.json";
pub const YAML_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.data.yaml";
pub const XML_IMPORTER_CAPABILITY: &str = "runtime.asset.importer.data.xml";

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[
        RUNTIME_CAPABILITY,
        TOML_IMPORTER_CAPABILITY,
        JSON_IMPORTER_CAPABILITY,
        YAML_IMPORTER_CAPABILITY,
        XML_IMPORTER_CAPABILITY,
    ]
}

pub fn supported_targets() -> [RuntimeTargetMode; 2] {
    [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ]
}

pub fn supported_platforms() -> [ExportTargetPlatform; 3] {
    [
        ExportTargetPlatform::Windows,
        ExportTargetPlatform::Linux,
        ExportTargetPlatform::Macos,
    ]
}

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(MODULE_NAME, "Data asset importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        descriptor("asset_importer.data.toml", ["toml"])
            .with_required_capabilities([TOML_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.data.json", ["json"])
            .with_required_capabilities([JSON_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.data.yaml", ["yaml", "yml"])
            .with_required_capabilities([YAML_IMPORTER_CAPABILITY]),
        descriptor("asset_importer.data.xml", ["xml"])
            .with_required_capabilities([XML_IMPORTER_CAPABILITY]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "Data Asset Importers")
        .with_category("asset_importer")
        .with_supported_targets(supported_targets())
        .with_supported_platforms(supported_platforms())
        .with_capabilities(runtime_capabilities().iter().copied())
        .with_runtime_module(runtime_module_manifest());
    for importer in asset_importer_descriptors() {
        manifest = manifest.with_asset_importer(importer);
    }
    manifest
}

pub fn runtime_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::runtime("asset_importer.data.runtime", RUNTIME_CRATE_NAME)
        .with_target_modes(supported_targets())
        .with_capabilities(runtime_capabilities().iter().copied())
}

pub fn runtime_selection() -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: PLUGIN_ID.to_string(),
        enabled: true,
        required: false,
        target_modes: supported_targets().to_vec(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: Some(RUNTIME_CRATE_NAME.to_string()),
        editor_crate: None,
        features: Vec::new(),
    }
}

pub fn plugin_registration() -> RuntimePluginRegistrationReport {
    let mut extensions = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::new();
    if let Err(error) = register_runtime_extensions(&mut extensions) {
        diagnostics.push(error.to_string());
    }
    RuntimePluginRegistrationReport {
        package_manifest: package_manifest(),
        project_selection: runtime_selection(),
        extensions,
        diagnostics,
    }
}

pub fn register_runtime_extensions(
    registry: &mut RuntimeExtensionRegistry,
) -> Result<(), RuntimeExtensionRegistryError> {
    registry.register_module(module_descriptor())?;
    for importer in asset_importer_descriptors() {
        let import_fn = match importer.id.as_str() {
            "asset_importer.data.toml" => import_toml_data,
            "asset_importer.data.json" => import_json_data,
            "asset_importer.data.yaml" => import_yaml_data,
            "asset_importer.data.xml" => import_xml_data,
            _ => unreachable!("asset_importer_descriptors returns only known data importer ids"),
        };
        registry.register_asset_importer(FunctionAssetImporter::new(importer, import_fn))?;
    }
    Ok(())
}

pub fn import_toml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let value: toml::Value = toml::from_str(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse toml data: {error}")))?;
    data_outcome(
        context,
        DataAssetFormat::Toml,
        text,
        serde_json::to_value(value)?,
    )
}

pub fn import_json_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let canonical_json: Value = serde_json::from_str(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse json data: {error}")))?;
    data_outcome(context, DataAssetFormat::Json, text, canonical_json)
}

pub fn import_yaml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let canonical_json: Value = serde_yaml::from_str(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse yaml data: {error}")))?;
    data_outcome(context, DataAssetFormat::Yaml, text, canonical_json)
}

pub fn import_xml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let document = roxmltree::Document::parse(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse xml data: {error}")))?;
    let canonical_json = xml_element_to_json(document.root_element());
    data_outcome(context, DataAssetFormat::Xml, text, canonical_json)
}

fn data_outcome(
    context: &AssetImportContext,
    format: DataAssetFormat,
    text: String,
    canonical_json: Value,
) -> Result<AssetImportOutcome, AssetImportError> {
    Ok(AssetImportOutcome::new(ImportedAsset::Data(DataAsset {
        uri: context.uri.clone(),
        format,
        text,
        canonical_json,
    })))
}

// XML is not intrinsically JSON-shaped, so the importer emits a stable neutral tree DTO.
fn xml_element_to_json(node: roxmltree::Node<'_, '_>) -> Value {
    let mut object = Map::new();
    object.insert(
        "name".to_string(),
        Value::String(node.tag_name().name().to_string()),
    );
    if let Some(namespace) = node.tag_name().namespace() {
        object.insert(
            "namespace".to_string(),
            Value::String(namespace.to_string()),
        );
    }

    let attributes = node
        .attributes()
        .map(|attribute| {
            (
                attribute.name().to_string(),
                Value::String(attribute.value().to_string()),
            )
        })
        .collect::<Map<_, _>>();
    if !attributes.is_empty() {
        object.insert("attributes".to_string(), Value::Object(attributes));
    }

    let text_nodes = node
        .children()
        .filter_map(|child| child.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(|text| Value::String(text.to_string()))
        .collect::<Vec<_>>();
    match text_nodes.as_slice() {
        [] => {}
        [text] => {
            object.insert("text".to_string(), text.clone());
        }
        _ => {
            object.insert("text".to_string(), Value::Array(text_nodes));
        }
    }

    let children = node
        .children()
        .filter(|child| child.is_element())
        .map(xml_element_to_json)
        .collect::<Vec<_>>();
    if !children.is_empty() {
        object.insert("children".to_string(), Value::Array(children));
    }

    Value::Object(object)
}

fn descriptor(
    id: impl Into<String>,
    extensions: impl IntoIterator<Item = impl Into<String>>,
) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::Data, 1)
        .with_priority(100)
        .with_source_extensions(extensions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn package_declares_data_importer_capabilities() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.source_extensions.contains(&"yaml".to_string())));
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest
            .capabilities
            .contains(&XML_IMPORTER_CAPABILITY.to_string()));
    }

    #[test]
    fn registration_contributes_module_and_data_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 4);
    }

    #[test]
    fn yaml_importer_decodes_data_asset() {
        let asset = import_fixture(
            "config.yaml",
            "name: zircon\nscale: 2\nitems:\n  - a\n  - b\n",
        );

        match asset {
            ImportedAsset::Data(data) => {
                assert_eq!(data.format, DataAssetFormat::Yaml);
                assert_eq!(data.canonical_json["name"], "zircon");
                assert_eq!(data.canonical_json["scale"], 2);
                assert_eq!(data.canonical_json["items"], json!(["a", "b"]));
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn xml_importer_decodes_data_asset() {
        let asset = import_fixture(
            "panel.xml",
            r#"<panel id="main"><label>Hello</label><button enabled="true">Run</button></panel>"#,
        );

        match asset {
            ImportedAsset::Data(data) => {
                assert_eq!(data.format, DataAssetFormat::Xml);
                assert_eq!(data.canonical_json["name"], "panel");
                assert_eq!(data.canonical_json["attributes"]["id"], "main");
                assert_eq!(data.canonical_json["children"][0]["name"], "label");
                assert_eq!(data.canonical_json["children"][0]["text"], "Hello");
                assert_eq!(
                    data.canonical_json["children"][1]["attributes"]["enabled"],
                    "true"
                );
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn invalid_yaml_returns_parse_error() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("broken.yaml"))
            .unwrap();
        let context = context_for("broken.yaml", "key: [unterminated");

        let error = importer.import(&context).unwrap_err();

        assert!(error.to_string().contains("parse yaml data"));
    }

    fn import_fixture(path: &str, source: &str) -> ImportedAsset {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new(path))
            .unwrap();
        importer
            .import(&context_for(path, source))
            .unwrap()
            .imported_asset
    }

    fn context_for(path: &str, source: &str) -> AssetImportContext {
        let file_name = path.replace('\\', "/");
        let uri = format!("res://data/{file_name}");
        AssetImportContext::new(
            path.into(),
            zircon_runtime::asset::AssetUri::parse(&uri).unwrap(),
            source.as_bytes().to_vec(),
            Default::default(),
        )
    }
}
