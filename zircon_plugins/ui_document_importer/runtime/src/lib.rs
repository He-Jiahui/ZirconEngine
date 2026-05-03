use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    AssetSchemaMigrationReport, FunctionAssetImporter, ImportedAsset, UiLayoutAsset, UiStyleAsset,
    UiWidgetAsset,
};
use zircon_runtime::core::ModuleDescriptor;
use zircon_runtime::ui::template::{
    UiAssetDocumentRuntimeExt, UiAssetLoader, UiAssetSchemaVersionPolicy,
    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::PluginModuleManifest,
    plugin::PluginPackageManifest, plugin::ProjectPluginSelection,
    plugin::RuntimeExtensionRegistry, plugin::RuntimeExtensionRegistryError,
    plugin::RuntimePluginRegistrationReport, RuntimeTargetMode,
};
use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetKind};

pub const PLUGIN_ID: &str = "ui_document_importer";
pub const RUNTIME_CRATE_NAME: &str = "zircon_plugin_ui_document_importer_runtime";
pub const MODULE_NAME: &str = "UiDocumentImporterModule";
pub const RUNTIME_CAPABILITY: &str = "runtime.plugin.ui_document_importer";
pub const IMPORTER_CAPABILITY: &str = "runtime.asset.importer.ui_document";
pub const UI_BINARY_MAGIC: &[u8; 8] = b"ZRUI001\0";
pub const UI_BINARY_FORMAT_VERSION: u32 = 1;

const UI_BINARY_HEADER_LEN: usize = UI_BINARY_MAGIC.len() + std::mem::size_of::<u32>();

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RUNTIME_CAPABILITY, IMPORTER_CAPABILITY]
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
    ModuleDescriptor::new(MODULE_NAME, "UI document importer plugin")
}

pub fn asset_importer_descriptors() -> Vec<AssetImporterDescriptor> {
    vec![
        ui_descriptor("ui_document_importer.typed_toml", 120).with_full_suffixes([".ui.toml"]),
        ui_descriptor("ui_document_importer.serialized_json", 110).with_full_suffixes([".ui.json"]),
        ui_descriptor("ui_document_importer.serialized_binary", 90)
            .with_source_extensions(["zui", "uidoc"]),
    ]
}

pub fn package_manifest() -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(PLUGIN_ID, "UI Document Importer")
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
    PluginModuleManifest::runtime("ui_document_importer.runtime", RUNTIME_CRATE_NAME)
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
        match importer.id.as_str() {
            "ui_document_importer.typed_toml" => registry.register_asset_importer(
                FunctionAssetImporter::new(importer, import_ui_toml_document),
            )?,
            "ui_document_importer.serialized_json" => registry.register_asset_importer(
                FunctionAssetImporter::new(importer, import_ui_json_document),
            )?,
            "ui_document_importer.serialized_binary" => registry.register_asset_importer(
                FunctionAssetImporter::new(importer, import_ui_binary_document),
            )?,
            _ => unreachable!("asset_importer_descriptors returns only known UI importer ids"),
        }
    }
    Ok(())
}

pub fn import_ui_toml_document(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let migration = UiAssetLoader::load_toml_str_with_migration_report(&document)
        .map_err(|error| AssetImportError::Parse(error.to_string()))?;
    let migration_report = AssetSchemaMigrationReport {
        source_schema_version: migration.report.source_schema_version,
        target_schema_version: migration.report.target_schema_version,
        summary: format!(
            "ui asset schema {:?}; steps: {:?}",
            migration.report.source_kind, migration.report.steps
        ),
    };

    ui_document_outcome(migration.document, migration_report)
}

pub fn import_ui_json_document(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document: UiAssetDocument =
        serde_json::from_slice(&context.source_bytes).map_err(|error| {
            AssetImportError::Parse(format!(
                "parse ui asset json {}: {error}",
                context.source_path.display()
            ))
        })?;
    import_serialized_ui_document(document, "json tree")
}

pub fn import_ui_binary_document(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = decode_ui_binary_document(&context.source_bytes, context)?;
    import_serialized_ui_document(document, "binary document")
}

pub fn encode_ui_binary_document(document: &UiAssetDocument) -> Result<Vec<u8>, AssetImportError> {
    let payload = bincode::serialize(document)
        .map_err(|error| AssetImportError::Parse(format!("encode ui binary document: {error}")))?;
    let mut bytes = Vec::with_capacity(UI_BINARY_HEADER_LEN + payload.len());
    bytes.extend_from_slice(UI_BINARY_MAGIC);
    bytes.extend_from_slice(&UI_BINARY_FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

fn decode_ui_binary_document(
    source_bytes: &[u8],
    context: &AssetImportContext,
) -> Result<UiAssetDocument, AssetImportError> {
    if source_bytes.len() < UI_BINARY_HEADER_LEN {
        return Err(AssetImportError::Parse(format!(
            "parse ui binary document {}: expected at least {UI_BINARY_HEADER_LEN} bytes",
            context.source_path.display()
        )));
    }
    if &source_bytes[..UI_BINARY_MAGIC.len()] != UI_BINARY_MAGIC {
        return Err(AssetImportError::Parse(format!(
            "parse ui binary document {}: invalid ZRUI001 magic header",
            context.source_path.display()
        )));
    }

    let version_offset = UI_BINARY_MAGIC.len();
    let container_version = u32::from_le_bytes([
        source_bytes[version_offset],
        source_bytes[version_offset + 1],
        source_bytes[version_offset + 2],
        source_bytes[version_offset + 3],
    ]);
    if container_version != UI_BINARY_FORMAT_VERSION {
        return Err(AssetImportError::SchemaMigration(format!(
            "ui binary document {} uses unsupported container version {}; current supported version is {}",
            context.source_path.display(),
            container_version,
            UI_BINARY_FORMAT_VERSION
        )));
    }

    bincode::deserialize(&source_bytes[UI_BINARY_HEADER_LEN..]).map_err(|error| {
        AssetImportError::Parse(format!(
            "decode ui binary document {}: {error}",
            context.source_path.display()
        ))
    })
}

fn import_serialized_ui_document(
    mut document: UiAssetDocument,
    source_label: &str,
) -> Result<AssetImportOutcome, AssetImportError> {
    let source_version = document.asset.version;
    if !UiAssetSchemaVersionPolicy::is_supported_source_schema(source_version) {
        return Err(AssetImportError::SchemaMigration(format!(
            "ui asset {} uses unsupported schema version {}; current supported version is {}",
            document.asset.id, source_version, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
        )));
    }

    document.asset.version = UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION;
    document
        .validate_tree_authority()
        .map_err(|error| AssetImportError::Parse(error.to_string()))?;

    let summary = if UiAssetSchemaVersionPolicy::requires_source_schema_migration(source_version) {
        format!(
            "ui asset schema {source_label}; source version bumped from {source_version} to {UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION}"
        )
    } else {
        format!("ui asset schema {source_label}; validated current version")
    };
    let migration_report = AssetSchemaMigrationReport {
        source_schema_version: Some(source_version),
        target_schema_version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
        summary,
    };

    ui_document_outcome(document, migration_report)
}

fn ui_document_outcome(
    document: UiAssetDocument,
    migration_report: AssetSchemaMigrationReport,
) -> Result<AssetImportOutcome, AssetImportError> {
    let outcome = match document.asset.kind {
        UiAssetKind::Layout => {
            AssetImportOutcome::new(ImportedAsset::UiLayout(UiLayoutAsset { document }))
        }
        UiAssetKind::Widget => {
            AssetImportOutcome::new(ImportedAsset::UiWidget(UiWidgetAsset { document }))
        }
        UiAssetKind::Style => {
            AssetImportOutcome::new(ImportedAsset::UiStyle(UiStyleAsset { document }))
        }
    };
    Ok(outcome.with_migration_report(migration_report))
}

fn ui_descriptor(id: impl Into<String>, priority: i32) -> AssetImporterDescriptor {
    AssetImporterDescriptor::new(id, PLUGIN_ID, AssetKind::UiLayout, 1)
        .with_priority(priority)
        .with_additional_output_kinds([AssetKind::UiWidget, AssetKind::UiStyle])
        .with_required_capabilities([IMPORTER_CAPABILITY])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_declares_ui_document_importers() {
        let manifest = package_manifest();

        assert_eq!(manifest.id, PLUGIN_ID);
        assert!(manifest
            .capabilities
            .contains(&RUNTIME_CAPABILITY.to_string()));
        assert!(manifest.asset_importers.iter().any(|importer| {
            importer.full_suffixes.contains(&".ui.toml".to_string())
                && importer.allows_output_kind(AssetKind::UiWidget)
        }));
        assert!(manifest.asset_importers.iter().any(|importer| importer
            .source_extensions
            .contains(&"zui".to_string())
            && importer.source_extensions.contains(&"uidoc".to_string())));
    }

    #[test]
    fn registration_contributes_module_and_importers() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == MODULE_NAME));
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 3);
    }

    #[test]
    fn typed_toml_importer_decodes_ui_layout_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("layout.ui.toml"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "layout.ui.toml".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/layout.ui.toml").unwrap(),
            br#"
[asset]
kind = "layout"
id = "main"
"#
            .to_vec(),
            Default::default(),
        );

        let imported = importer.import(&context).unwrap().imported_asset;

        assert!(matches!(
            imported,
            zircon_runtime::asset::ImportedAsset::UiLayout(_)
        ));
    }

    #[test]
    fn serialized_json_importer_decodes_ui_layout_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("layout.ui.json"))
            .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "layout.ui.json".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/layout.ui.json").unwrap(),
            br#"
{
  "asset": {
    "kind": "layout",
    "id": "json.layout",
    "version": 1,
    "display_name": "JSON Layout"
  },
  "root": {
    "node_id": "root",
    "type": "Panel"
  }
}
"#
            .to_vec(),
            Default::default(),
        );

        let outcome = importer.import(&context).unwrap();

        match outcome.imported_asset {
            zircon_runtime::asset::ImportedAsset::UiLayout(asset) => {
                assert_eq!(asset.document.asset.id, "json.layout");
                assert_eq!(asset.document.root_node_id(), Some("root"));
                assert_eq!(
                    outcome.migration_report.unwrap().target_schema_version,
                    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
                );
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn serialized_binary_importer_decodes_ui_layout_asset() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("layout.zui"))
            .unwrap();
        let document: UiAssetDocument = serde_json::from_slice(
            br#"
{
  "asset": {
    "kind": "layout",
    "id": "binary.layout",
    "version": 1,
    "display_name": "Binary Layout"
  },
  "root": {
    "node_id": "root",
    "type": "Panel"
  }
}
"#,
        )
        .unwrap();
        let context = zircon_runtime::asset::AssetImportContext::new(
            "layout.zui".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/layout.zui").unwrap(),
            encode_ui_binary_document(&document).unwrap(),
            Default::default(),
        );

        let outcome = importer.import(&context).unwrap();

        match outcome.imported_asset {
            zircon_runtime::asset::ImportedAsset::UiLayout(asset) => {
                assert_eq!(asset.document.asset.id, "binary.layout");
                assert_eq!(asset.document.root_node_id(), Some("root"));
                assert_eq!(
                    outcome.migration_report.unwrap().target_schema_version,
                    UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
                );
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }

    #[test]
    fn serialized_binary_rejects_invalid_magic() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("broken.uidoc"))
            .unwrap();
        let mut source = vec![0; UI_BINARY_HEADER_LEN];
        source.extend_from_slice(b"payload");
        let context = zircon_runtime::asset::AssetImportContext::new(
            "broken.uidoc".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/broken.uidoc").unwrap(),
            source,
            Default::default(),
        );

        let error = importer.import(&context).unwrap_err();

        assert!(error.to_string().contains("invalid ZRUI001 magic header"));
    }

    #[test]
    fn serialized_binary_rejects_future_container_version() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("future.zui"))
            .unwrap();
        let mut source = Vec::new();
        source.extend_from_slice(UI_BINARY_MAGIC);
        source.extend_from_slice(&(UI_BINARY_FORMAT_VERSION + 1).to_le_bytes());
        let context = zircon_runtime::asset::AssetImportContext::new(
            "future.zui".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/future.zui").unwrap(),
            source,
            Default::default(),
        );

        let error = importer.import(&context).unwrap_err();

        assert!(error.to_string().contains("unsupported container version"));
    }

    #[test]
    fn serialized_json_rejects_future_schema() {
        let report = plugin_registration();
        let importer = report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("future.ui.json"))
            .unwrap();
        let source = format!(
            r#"{{
  "asset": {{
    "kind": "layout",
    "id": "future.layout",
    "version": {}
  }},
  "root": {{
    "node_id": "root"
  }}
}}"#,
            UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION + 1
        );
        let context = zircon_runtime::asset::AssetImportContext::new(
            "future.ui.json".into(),
            zircon_runtime::asset::AssetUri::parse("res://ui/future.ui.json").unwrap(),
            source.into_bytes(),
            Default::default(),
        );

        let error = importer.import(&context).unwrap_err();

        assert!(error.to_string().contains("unsupported schema version"));
    }
}
