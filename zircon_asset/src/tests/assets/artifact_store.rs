use zircon_resource::ResourceRecord;

use std::fs;

use crate::project::ProjectPaths;
use crate::tests::project::unique_temp_project_root;
use crate::{
    AlphaMode, ArtifactStore, AssetId, AssetKind, AssetReference, AssetUri, ImportedAsset,
    MaterialAsset,
};

#[test]
fn artifact_store_roundtrips_material_assets_in_library() {
    let root = unique_temp_project_root("artifact_store");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let material = MaterialAsset {
        name: Some("Grid".to_string()),
        shader: asset_reference("res://shaders/pbr.wgsl"),
        base_color: [0.8, 0.7, 0.6, 1.0],
        base_color_texture: Some(asset_reference("res://textures/grid.png")),
        normal_texture: None,
        metallic: 0.2,
        roughness: 0.7,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Material,
        AssetUri::parse("res://materials/grid.material.toml").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(
            &paths,
            &metadata,
            &ImportedAsset::Material(material.clone()),
        )
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_eq!(loaded, ImportedAsset::Material(material));

    let _ = fs::remove_dir_all(root);
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
