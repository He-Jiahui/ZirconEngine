use crate::{
    AssetReference, AssetUri, AssetUuid, SceneAsset, SceneCameraAsset, SceneDirectionalLightAsset,
    SceneEntityAsset, SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};

#[test]
fn scene_asset_toml_roundtrip_preserves_entities_and_bindings() {
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 1,
                name: "Camera".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 2.0, 5.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    fov_y_radians: 1.0471976,
                    z_near: 0.1,
                    z_far: 200.0,
                }),
                mesh: None,
                directional_light: None,
            },
            SceneEntityAsset {
                entity: 2,
                name: "Model".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: AssetReference::new(
                        AssetUuid::from_stable_label("robot-model"),
                        AssetUri::parse("res://models/robot.gltf").unwrap(),
                    ),
                    material: AssetReference::new(
                        AssetUuid::from_stable_label("robot-material"),
                        AssetUri::parse("res://materials/robot.material.toml").unwrap(),
                    ),
                }),
                directional_light: None,
            },
            SceneEntityAsset {
                entity: 3,
                name: "Sun".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 4.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0004,
                mobility: SceneMobilityAsset::Static,
                camera: None,
                mesh: None,
                directional_light: Some(SceneDirectionalLightAsset {
                    direction: [-0.4, -1.0, -0.25],
                    color: [1.0, 1.0, 1.0],
                    intensity: 3.0,
                }),
            },
        ],
    };

    let document = scene.to_toml_string().unwrap();
    let loaded = SceneAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, scene);
}

#[test]
fn scene_asset_parses_legacy_locator_only_mesh_bindings() {
    let document = r#"
[[entities]]
entity = 2
name = "Model"
parent = 0
active = true
transform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }
mesh = { model = "res://models/robot.gltf", material = "res://materials/robot.material.toml" }
"#;

    let loaded = SceneAsset::from_toml_str(document).unwrap();
    let mesh = loaded.entities[0].mesh.as_ref().unwrap();

    assert_eq!(
        mesh.model.locator,
        AssetUri::parse("res://models/robot.gltf").unwrap()
    );
    assert_eq!(
        mesh.material.locator,
        AssetUri::parse("res://materials/robot.material.toml").unwrap()
    );
}

#[test]
fn scene_asset_defaults_new_runtime_foundation_fields_when_omitted() {
    let document = r#"
[[entities]]
entity = 7
name = "Legacy"
transform = { translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }
active = true
"#;

    let loaded = SceneAsset::from_toml_str(document).unwrap();
    let entity = &loaded.entities[0];

    assert!(entity.active);
    assert_eq!(entity.render_layer_mask, 0x0000_0001);
    assert_eq!(entity.mobility, SceneMobilityAsset::Dynamic);
}
