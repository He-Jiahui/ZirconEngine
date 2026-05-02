use crate::asset::{
    AssetReference, AssetUri, ImportedAsset, MaterialGraphAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset, PrefabAsset, SceneAsset, SceneEntityAsset, TerrainAsset,
    TerrainLayerAsset, TileMapAsset, TileMapLayerAsset, TileMapProjectionAsset, TileSetAsset,
    TileSetTileAsset, TransformAsset,
};

#[test]
fn authoring_assets_roundtrip_and_collect_references() {
    let material = reference("res://materials/terrain.material.toml");
    let weightmap = reference("res://terrain/grass.png");
    let terrain = TerrainAsset {
        uri: AssetUri::parse("res://terrain/island.terrain.toml").unwrap(),
        name: "Island".to_string(),
        width: 2,
        height: 2,
        sample_spacing: 1.0,
        height_scale: 8.0,
        height_samples: vec![0.0, 1.0, 0.25, 0.5],
        layers: vec![TerrainLayerAsset {
            name: "Grass".to_string(),
            material: Some(material.clone()),
            weightmap: Some(weightmap.clone()),
            strength: 1.0,
        }],
    };

    terrain.validate_dimensions().unwrap();
    let encoded = terrain.to_toml_string().unwrap();
    let decoded = TerrainAsset::from_toml_str(&encoded).unwrap();
    assert_eq!(decoded, terrain);
    assert_eq!(decoded.direct_references(), vec![material, weightmap]);
}

#[test]
fn tilemap_asset_validates_projection_and_layer_sizes() {
    let tile_set = reference("res://tiles/world.tileset.toml");
    let tilemap = TileMapAsset {
        uri: AssetUri::parse("res://tiles/world.tilemap.toml").unwrap(),
        width: 2,
        height: 2,
        projection: TileMapProjectionAsset::HexagonalStaggered,
        tile_set: tile_set.clone(),
        layers: vec![TileMapLayerAsset {
            name: "Ground".to_string(),
            visible: true,
            opacity: 1.0,
            tiles: vec![Some(1), None, Some(2), Some(3)],
        }],
    };
    let tileset = TileSetAsset {
        uri: AssetUri::parse("res://tiles/world.tileset.toml").unwrap(),
        tile_width: 16,
        tile_height: 16,
        image: reference("res://tiles/world.png"),
        tiles: vec![TileSetTileAsset {
            id: 1,
            name: Some("Grass".to_string()),
            collider: None,
        }],
    };

    tilemap.validate_layers().unwrap();
    assert_eq!(
        tilemap.projection,
        TileMapProjectionAsset::HexagonalStaggered
    );
    assert_eq!(
        ImportedAsset::TileMap(tilemap.clone()).direct_references(),
        vec![tile_set]
    );
    assert_eq!(tileset.direct_references().len(), 1);
}

#[test]
fn material_graph_requires_output_node_and_reports_references() {
    let shader = reference("res://shaders/pbr.wgsl");
    let texture = reference("res://textures/albedo.png");
    let graph = MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/hero.material_graph.toml").unwrap(),
        name: "Hero".to_string(),
        shader: Some(shader.clone()),
        nodes: vec![
            MaterialGraphNodeAsset {
                id: "albedo".to_string(),
                position: [0.0, 0.0],
                kind: MaterialGraphNodeKindAsset::TextureSample {
                    texture: texture.clone(),
                },
            },
            MaterialGraphNodeAsset {
                id: "output".to_string(),
                position: [240.0, 0.0],
                kind: MaterialGraphNodeKindAsset::Output,
            },
        ],
        links: Vec::new(),
        parameters: Default::default(),
    };

    graph.validate_output_node().unwrap();
    assert_eq!(graph.direct_references(), vec![shader, texture]);
}

#[test]
fn prefab_asset_collects_scene_references_without_editor_state() {
    let material = reference("res://materials/default.material.toml");
    let model = reference("res://models/cube.model.toml");
    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 1,
            name: "Cube".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 1,
            mobility: Default::default(),
            camera: None,
            mesh: Some(crate::asset::SceneMeshInstanceAsset {
                model: model.clone(),
                material: material.clone(),
            }),
            directional_light: None,
            point_light: None,
            spot_light: None,
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
        }],
    };
    let prefab = PrefabAsset {
        uri: AssetUri::parse("res://prefabs/cube.prefab.toml").unwrap(),
        name: "Cube".to_string(),
        scene,
        exposed_properties: vec!["Cube.transform".to_string()],
    };

    assert_eq!(prefab.direct_references(), vec![model, material]);
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
