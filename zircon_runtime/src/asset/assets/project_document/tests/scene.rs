use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use super::*;
use crate::asset::assets::{
    PrefabInstanceAsset, PrefabPropertyOverrideAsset, SceneComponentAssetRecord, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};
use crate::asset::{AssetUri, AssetUuid};

#[test]
fn formal_scene_writer_reader_round_trips_project_builtin_and_subasset_references() {
    let model_guid: AssetUuid = "fa111111-2222-4333-8444-555555555555".parse().unwrap();
    let mesh_guid: AssetUuid = "fb111111-2222-4333-8444-555555555555".parse().unwrap();
    let model = AssetReference::new(
        model_guid,
        AssetUri::parse("res://models/hero.glb").unwrap(),
    );
    let mesh = AssetReference::new(
        mesh_guid,
        AssetUri::parse("res://models/hero.glb#Mesh0").unwrap(),
    );
    let material =
        AssetReference::from_locator(AssetUri::parse("builtin://material/default").unwrap());
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 1,
            name: "Roundtrip".to_owned(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 1,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: Some(SceneMeshInstanceAsset {
                model: model.clone(),
                mesh: Some(mesh.clone()),
                material: material.clone(),
                render_queue: 0,
                material_queue: 0,
                order_in_layer: 0,
                depth_bias: 0.0,
                morph_weights: Vec::new(),
                primitives: Vec::new(),
                lods: Vec::new(),
            }),
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            post_process_volume: None,
            rigid_body: None,
            collider: None,
            joint: None,
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
            terrain: None,
            tilemap: None,
            prefab_instance: None,
            components: vec![SceneComponentAssetRecord::from_typed(
                "tests.scene.custom_probe",
                "tests.scene.custom_probe.v1",
                1,
                "tests.scene.provider",
                &serde_json::json!({
                    "enabled": true,
                    "references": [mesh.clone()],
                    "weight": 0.75
                }),
            )
            .unwrap()],
            script_bindings: Vec::new(),
        }],
    };

    let document = serialize_scene(&scene, |reference| {
        if reference.locator.scheme() == ResourceScheme::Builtin {
            return Ok(PersistedAssetReference::builtin(reference.locator.clone()));
        }
        Ok(PersistedAssetReference::project(
            AssetRef::try_new(
                reference.uuid,
                RelPath::parse("models/hero.glb").unwrap(),
                reference.locator.label().map(str::to_owned),
            )
            .unwrap(),
        ))
    })
    .unwrap();
    let reloaded = deserialize_scene(&document, |reference| {
        if let Some(locator) = reference.builtin_locator() {
            return Ok(AssetReference::from_locator(locator.clone()));
        }
        let reference = reference.project_ref().expect("project reference");
        let mut locator = format!("res://{}", reference.path_hint());
        if let Some(sub) = reference.sub() {
            locator.push('#');
            locator.push_str(sub);
        }
        Ok(AssetReference::new(
            reference.guid(),
            AssetUri::parse(&locator).unwrap(),
        ))
    })
    .unwrap();

    assert!(document.contains("tests.scene.custom_probe"));
    assert_eq!(
        reloaded.entities[0].components,
        scene.entities[0].components
    );
    assert_eq!(reloaded, scene);
}

#[test]
fn formal_scene_writer_reader_preserves_prefab_instance_metadata() {
    let prefab_guid: AssetUuid = "fc111111-2222-4333-8444-555555555555".parse().unwrap();
    let prefab = AssetReference::new(
        prefab_guid,
        AssetUri::parse("res://prefabs/hero.prefab.toml").unwrap(),
    );
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 7,
            name: "HeroInstance".to_owned(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 1,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: None,
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            post_process_volume: None,
            rigid_body: None,
            collider: None,
            joint: None,
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
            terrain: None,
            tilemap: None,
            prefab_instance: Some(PrefabInstanceAsset {
                prefab: prefab.clone(),
                local_transform: TransformAsset {
                    translation: [3.0, 2.0, 1.0],
                    ..Default::default()
                },
                overrides: vec![PrefabPropertyOverrideAsset {
                    entity_path: "Root/Weapon".to_owned(),
                    property_path: "material.tint".to_owned(),
                    value: serde_json::json!({
                        "enabled": true,
                        "color": [1.0, 0.5, 0.25, 1.0],
                    }),
                }],
            }),
            components: Vec::new(),
            script_bindings: Vec::new(),
        }],
    };

    let document = serialize_scene(&scene, |reference| {
        Ok(PersistedAssetReference::project(
            AssetRef::try_new(
                reference.uuid,
                RelPath::parse("prefabs/hero.prefab.toml").unwrap(),
                reference.locator.label().map(str::to_owned),
            )
            .unwrap(),
        ))
    })
    .unwrap();
    let reloaded = deserialize_scene(&document, |reference| {
        let reference = reference.project_ref().expect("project reference");
        Ok(AssetReference::new(
            reference.guid(),
            AssetUri::parse(&format!("res://{}", reference.path_hint())).unwrap(),
        ))
    })
    .unwrap();

    assert_eq!(reloaded, scene);
}
