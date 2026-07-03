use std::{fs, path::Path};

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetKind, AssetMetaDocument, AssetSourceUnit, AssetUri, AssetUuid, DataAsset, DataAssetFormat,
    FunctionAssetImporter, ImportedAsset, ImportedAssetEntry, MaterialAsset, ProjectManager,
    ProjectManifest, ProjectPaths, ShaderAsset, ShaderOptionAsset, ShaderSourceFileAsset,
    ShaderSourceLanguage, ShaderTextureSlotAsset, ZShaderDocumentV2, ZShaderV2Error,
};
use crate::core::framework::render::ShaderAssetKind;
use crate::core::resource::ResourceState;
use crate::plugin::PluginPackageManifest;

mod compound_shader;
mod metadata_lifecycle;
mod package_roots;
mod shader_diagnostics_fixture;

fn multi_asset_importer(extension: &'static str) -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new("test.multi.bundle", "test.multi", AssetKind::Data, 1)
            .with_source_extensions([extension])
            .with_additional_output_kinds([AssetKind::Texture]),
        import_multi_asset_bundle,
    )
}

fn material_for_shader(shader_uri: &AssetUri) -> MaterialAsset {
    MaterialAsset {
        name: Some("UnlitMaterial".to_string()),
        shader: crate::asset::AssetReference::from_locator(shader_uri.clone()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: crate::asset::AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn flaky_multi_asset_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new("test.multi.flaky", "test.multi", AssetKind::Data, 1)
            .with_source_extensions(["flaky"])
            .with_additional_output_kinds([AssetKind::Texture]),
        import_flaky_multi_asset_bundle,
    )
}

fn import_multi_asset_bundle(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    import_multi_asset_bundle_with_text(context, context.source_text()?)
}

fn import_flaky_multi_asset_bundle(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    if text == "fail" {
        return Err(AssetImportError::Parse(
            "transient flaky import failure".to_string(),
        ));
    }
    import_multi_asset_bundle_with_text(context, text)
}

fn import_multi_asset_bundle_with_text(
    context: &AssetImportContext,
    text: String,
) -> Result<AssetImportOutcome, AssetImportError> {
    let texture_uri = AssetUri::parse(&format!("{}#Texture0", context.uri)).unwrap();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json: serde_json::json!({ "bundle": true }),
        }),
    )
    .with_entry(ImportedAssetEntry::new(
        texture_uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            texture_uri,
            1,
            1,
            vec![255, 0, 255, 255],
        )),
    )))
}
