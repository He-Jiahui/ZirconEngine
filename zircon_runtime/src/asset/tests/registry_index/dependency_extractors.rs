use std::path::Path;

use crate::asset::registry::dependency_extractors::append_handwritten_dependencies;
use crate::asset::{
    AssetImportOutcome, AssetReference, ImportedAsset, MaterialAsset, ModelAsset, SceneAsset,
    UiV2ViewAsset,
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

#[test]
fn handwritten_ui_document_extractor_emits_import_and_resource_dependencies() {
    let view = UiV2ViewAsset::from_toml_str(
        r#"
[asset]
kind = "view"
id = "runtime.ui.dependency_test"
version = 2

[imports]
widgets = ["res://ui/common/button.zui#ToolbarButton"]
styles = ["res://ui/theme/editor.zui"]
resources = [
  { kind = "font", uri = "res://fonts/inter.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
]

[root]
node = "root"

[nodes.root]
component = "Text"
"#,
    )
    .expect("valid UI document fixture");

    let mut outcome = AssetImportOutcome::new(
        uri("res://ui/dependency_test.zui"),
        ImportedAsset::UiV2View(view),
    );
    append_handwritten_dependencies(&mut outcome);

    let dependencies = &outcome.entries[0].dependencies;
    assert_eq!(
        dependencies,
        &vec![
            uri("res://ui/common/button.zui"),
            uri("res://ui/theme/editor.zui"),
            uri("res://fonts/inter.font.toml"),
            uri("res://fonts/system.ttf"),
        ]
    );
}
