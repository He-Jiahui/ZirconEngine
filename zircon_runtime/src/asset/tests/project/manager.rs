use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::asset::project::AssetMetaDocument;
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::binary_artifact_cache_assertions::{
    assert_artifact_cache_files_are_zassets, assert_binary_artifact_cache,
};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    sample_animation_sequence_asset, sample_physics_material_asset, sample_sound_asset,
    write_checker_png, write_default_animation_clip, write_default_animation_graph,
    write_default_animation_sequence, write_default_animation_skeleton,
    write_default_animation_state_machine, write_default_material, write_default_physics_material,
    write_default_scene, write_test_wav, write_triangle_obj, write_valid_wgsl,
};
use crate::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetKind, AssetUri, AssetUuid, DataAsset, DataAssetFormat, FunctionAssetImporter,
    ImportedAsset, ImportedAssetEntry,
};
use crate::core::resource::ResourceState;

static COUNTED_IMPORT_CALLS: AtomicUsize = AtomicUsize::new(0);

mod artifact_cache_imports;
mod catalog_input_generation;
mod restore_failure_migration;
mod subassets_errors;
mod targeted_import;

fn counted_data_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new("test.counted.data", "test.counted", AssetKind::Data, 1)
            .with_source_extensions(["counted"]),
        import_counted_data,
    )
}

fn project_manager_with_first_wave_plugin_fixtures(root: impl AsRef<Path>) -> ProjectManager {
    let mut manager = ProjectManager::open(root).unwrap();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    manager
}

fn import_counted_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    COUNTED_IMPORT_CALLS.fetch_add(1, Ordering::SeqCst);
    let text = context.source_text()?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json: serde_json::json!({ "counted": true }),
        }),
    ))
}

fn import_material_with_dependencies(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Material(crate::asset::MaterialAsset {
            name: Some("Grid".to_string()),
            shader: crate::asset::AssetReference::from_locator(
                AssetUri::parse("builtin://shader/pbr.wgsl").unwrap(),
            ),
            parent: None,
            base_color: [0.8, 0.8, 0.8, 1.0],
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
            options: Default::default(),
            queue: None,
            validation_diagnostics: Vec::new(),
        }),
    )
    .with_dependency(AssetUri::parse("res://textures/checker.deptex").unwrap())
    .with_dependency(AssetUri::parse("res://textures/missing.deptex").unwrap()))
}

fn import_texture_dependency(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            context.uri.clone(),
            1,
            1,
            vec![255, 255, 255, 255],
        )),
    ))
}

fn import_multi_asset_bundle(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
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

fn import_duplicate_label_bundle(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let texture_uri = AssetUri::parse(&format!("{}#Texture0", context.uri)).unwrap();
    let duplicate_uri = texture_uri.clone();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text: "duplicate".to_string(),
            canonical_json: serde_json::json!({ "duplicate": true }),
        }),
    )
    .with_entry(ImportedAssetEntry::new(
        texture_uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            texture_uri,
            1,
            1,
            vec![255, 0, 0, 255],
        )),
    ))
    .with_entry(ImportedAssetEntry::new(
        duplicate_uri.clone(),
        ImportedAsset::Texture(crate::asset::TextureAsset::new_rgba8(
            duplicate_uri,
            1,
            1,
            vec![0, 255, 0, 255],
        )),
    )))
}
