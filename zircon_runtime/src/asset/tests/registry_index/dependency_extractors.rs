use std::path::Path;

use crate::asset::registry::dependency_extractors::append_handwritten_dependencies;
use crate::asset::{
    AssetImportOutcome, AssetReference, ImportedAsset, MaterialAsset, ModelAsset, SceneAsset,
};

use super::uri;

#[test]
fn handwritten_scene_material_and_model_extractors_emit_direct_dependencies() {
    let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/vampire/assets");
    let scene = SceneAsset::from_toml_str(
        &std::fs::read_to_string(project.join("scenes/main.scene.toml")).unwrap(),
    )
    .unwrap();
    let material = MaterialAsset::from_toml_str(
        &std::fs::read_to_string(project.join("materials/jungle_ground.zmaterial")).unwrap(),
    )
    .unwrap();
    let mut model = ModelAsset::from_toml_str(
        &std::fs::read_to_string(project.join("models/arena_floor.model.toml")).unwrap(),
    )
    .unwrap();
    model.primitives[0].mesh = Some(AssetReference::from_locator(uri(
        "res://models/arena_floor.model.toml#Mesh0/Primitive0",
    )));

    for (path, asset) in [
        ("res://scenes/main.scene.toml", ImportedAsset::Scene(scene)),
        (
            "res://materials/jungle_ground.zmaterial",
            ImportedAsset::Material(material),
        ),
        (
            "res://models/arena_floor.model.toml",
            ImportedAsset::Model(model),
        ),
    ] {
        let mut outcome = AssetImportOutcome::new(uri(path), asset);
        append_handwritten_dependencies(&mut outcome);
        assert!(
            !outcome.entries[0].dependencies.is_empty(),
            "{path} should expose direct dependency metadata"
        );
    }
}
