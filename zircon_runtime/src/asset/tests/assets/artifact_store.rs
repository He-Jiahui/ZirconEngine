use std::collections::BTreeMap;
use std::fs;

use crate::core::framework::physics::{PhysicsJointConstraintMetadata, PhysicsMaterialMetadata};
use crate::core::framework::render::RenderShaderDefinitionValue;
use crate::core::resource::ResourceRecord;

use crate::asset::project::ProjectPaths;
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    sample_animation_sequence_asset, sample_physics_material_asset,
};
use crate::asset::{
    AlphaMode, ArtifactStore, AssetId, AssetKind, AssetReference, AssetUri, DataAsset,
    DataAssetFormat, ImportedAsset, MaterialAsset, MeshAsset, MeshAttributeValues, MeshIndices,
    SceneAsset, SceneCameraAsset, SceneCameraTargetAsset, SceneColliderAsset,
    SceneColliderShapeAsset, SceneEntityAsset, SceneJointAsset, SceneJointKindAsset,
    SceneMobilityAsset, SceneScriptBindingAsset, ShaderAsset, ShaderMaterialPropertyAsset,
    ShaderSourceLanguage, TextureAsset, TransformAsset, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::render::RenderMeshTopology;

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
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Material,
        AssetUri::parse("res://materials/grid.zmaterial").unwrap(),
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

    assert!(artifact_uri.to_string().ends_with(".zasset"));
    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Material(material));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_material_assets_with_dynamic_property_values() {
    let root = unique_temp_project_root("artifact_store_material_dynamic_values");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let material = MaterialAsset::from_toml_str(
        r#"
version = 1
name = "Ghost Mist"

[shader]
uuid = "00000000-0000-0000-0000-000000000143"
url = "res://shaders/vampire_effect"

[overrides]
base_color = [0.42, 0.72, 0.86, 0.98]
metallic = 0.0
roughness = 0.9
emissive = [0.08, 0.18, 0.24]
double_sided = true

[overrides.alpha_mode]
mode = "opaque"
"#,
    )
    .unwrap();
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Material,
        AssetUri::parse("res://materials/ghost_mist.zmaterial").unwrap(),
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

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Material(material));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_data_assets_with_dynamic_json_values() {
    let root = unique_temp_project_root("artifact_store_data_dynamic_json");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let data = DataAsset {
        uri: AssetUri::parse("res://data/balance.json").unwrap(),
        format: DataAssetFormat::Json,
        text: r#"{"player":{"hp":120,"speed":5.5},"tags":["vampire",true,null]}"#.to_string(),
        canonical_json: serde_json::json!({
            "player": { "hp": 120, "speed": 5.5 },
            "tags": ["vampire", true, null],
            "spawn_count": 12_u64
        }),
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Data,
        AssetUri::parse("res://data/balance.json").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Data(data.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Data(data));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_scene_assets_with_script_binding_json_values() {
    let root = unique_temp_project_root("artifact_store_scene_dynamic_json");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 2,
            name: "Player".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: u32::MAX,
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
            prefab_instance: None,
            script_bindings: vec![SceneScriptBindingAsset {
                package: "vampire_game".to_string(),
                module: "main".to_string(),
                enabled: true,
                update: true,
                fixed_update: true,
                properties: BTreeMap::from([
                    ("role".to_string(), serde_json::json!("player")),
                    ("hp".to_string(), serde_json::json!(120.0)),
                    (
                        "loadout".to_string(),
                        serde_json::json!({
                            "weapon": "blood_bolt",
                            "pierce": 1,
                            "tags": ["starter", true, null]
                        }),
                    ),
                ]),
            }],
        }],
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Scene,
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Scene(scene.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Scene(scene));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_scene_assets_with_mesh_references() {
    let root = unique_temp_project_root("artifact_store_scene_mesh_references");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 338_863_232_448_440,
            name: "fence-gate".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 1,
            mobility: SceneMobilityAsset::Static,
            camera: None,
            mesh: Some(crate::asset::SceneMeshInstanceAsset {
                model: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Mesh0"),
                mesh: None,
                material: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Material0"),
                render_queue: 0,
                material_queue: 0,
                order_in_layer: 0,
                depth_bias: 0.0,
                morph_weights: Vec::new(),
                primitives: vec![crate::asset::SceneMeshPrimitiveBindingAsset {
                    mesh: asset_reference(
                        "res://models/kenney_graveyard/fence-gate.glb#Mesh0/Primitive0",
                    ),
                    material: asset_reference(
                        "res://models/kenney_graveyard/fence-gate.glb#Material0",
                    ),
                }],
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
            script_bindings: Vec::new(),
        }],
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Scene,
        AssetUri::parse("res://models/kenney_graveyard/fence-gate.glb#Scene0").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Scene(scene.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Scene(scene));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_scene_assets_with_camera_targets() {
    let root = unique_temp_project_root("artifact_store_scene_camera_targets");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 1,
            name: "RenderTargetCamera".to_string(),
            parent: None,
            transform: TransformAsset::default(),
            active: true,
            render_layer_mask: 1,
            mobility: SceneMobilityAsset::Dynamic,
            camera: Some(SceneCameraAsset {
                target: SceneCameraTargetAsset::Texture {
                    texture: asset_reference("res://textures/reflection_target.texture.toml"),
                },
                ..SceneCameraAsset::default()
            }),
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
            prefab_instance: None,
            script_bindings: Vec::new(),
        }],
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Scene,
        AssetUri::parse("res://scenes/render_target.scene.toml").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Scene(scene.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Scene(scene));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_scene_assets_with_physics_components() {
    let root = unique_temp_project_root("artifact_store_scene_physics_components");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let scene = SceneAsset {
        entities: vec![SceneEntityAsset {
            entity: 2,
            name: "PhysicsDoor".to_string(),
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
            collider: Some(SceneColliderAsset {
                shape: SceneColliderShapeAsset::Capsule {
                    radius: 0.35,
                    half_height: 1.1,
                },
                sensor: false,
                layer: 2,
                collision_group: 3,
                collision_mask: 0b101,
                material: Some(asset_reference(
                    "res://physics/materials/hinge.physics_material.toml",
                )),
                material_override: Some(PhysicsMaterialMetadata {
                    static_friction: 0.8,
                    dynamic_friction: 0.6,
                    restitution: 0.1,
                    ..PhysicsMaterialMetadata::default()
                }),
                local_transform: TransformAsset::default(),
            }),
            joint: Some(SceneJointAsset {
                joint_type: SceneJointKindAsset::Generic6Dof,
                connected_entity: Some(1),
                anchor: [0.0, 1.0, 0.0],
                axis: [0.0, 1.0, 0.0],
                limits: Some([-0.5, 0.5]),
                collide_connected: true,
                constraint: PhysicsJointConstraintMetadata {
                    linear_limits: [Some([-0.2, 0.2]), None, Some([0.0, 1.0])],
                    angular_limits: [None, Some([-0.25, 0.25]), None],
                    break_force: Some(120.0),
                    ..PhysicsJointConstraintMetadata::default()
                },
                skeleton_binding: None,
            }),
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
            terrain: None,
            tilemap: None,
            prefab_instance: None,
            script_bindings: Vec::new(),
        }],
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Scene,
        AssetUri::parse("res://scenes/physics.scene.toml").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Scene(scene.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Scene(scene));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_bincode_roundtrips_asset_reference() {
    let reference = asset_reference("res://models/kenney_graveyard/fence-gate.glb#Mesh0");

    let bytes = bincode::serialize(&reference).unwrap();
    let loaded = bincode::deserialize::<AssetReference>(&bytes).unwrap();

    assert_eq!(loaded, reference);
}

#[test]
fn artifact_store_bincode_roundtrips_scene_mesh_instance_asset() {
    let mesh = crate::asset::SceneMeshInstanceAsset {
        model: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Mesh0"),
        mesh: None,
        material: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Material0"),
        render_queue: 0,
        material_queue: 0,
        order_in_layer: 0,
        depth_bias: 0.0,
        morph_weights: Vec::new(),
        primitives: vec![crate::asset::SceneMeshPrimitiveBindingAsset {
            mesh: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Mesh0/Primitive0"),
            material: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Material0"),
        }],
        lods: Vec::new(),
    };

    let bytes = bincode::serialize(&mesh).unwrap();
    let loaded = bincode::deserialize::<crate::asset::SceneMeshInstanceAsset>(&bytes).unwrap();

    assert_eq!(loaded, mesh);
}

#[test]
fn artifact_store_roundtrips_mesh_assets_with_binary_attribute_payloads() {
    let root = unique_temp_project_root("artifact_store_mesh_binary_payloads");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/arena_floor.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [-1.0, 0.0, -1.0],
                    [1.0, 0.0, -1.0],
                    [0.0, 0.0, 1.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 1.0, 0.0]; 3]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]]),
            ),
        ]),
        indices: Some(MeshIndices::U32(vec![0, 1, 2])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Mesh,
        AssetUri::parse("res://meshes/arena_floor.zmesh").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Mesh(mesh.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Mesh(mesh));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_texture_assets_with_binary_payloads() {
    let root = unique_temp_project_root("artifact_store_texture_binary_payloads");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/jungle_ground_albedo.png").unwrap(),
        2,
        2,
        vec![
            48, 70, 36, 255, 54, 78, 42, 255, 42, 64, 34, 255, 68, 88, 45, 255,
        ],
    );
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Texture,
        AssetUri::parse("res://textures/jungle_ground_albedo.png").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Texture(texture.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Texture(texture));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_physics_material_assets_in_library() {
    let root = unique_temp_project_root("artifact_store_physics_material");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let physics_material = sample_physics_material_asset();
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::PhysicsMaterial,
        AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(
            &paths,
            &metadata,
            &ImportedAsset::PhysicsMaterial(physics_material.clone()),
        )
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert_eq!(
        artifact_uri.to_string().contains("physics/materials/"),
        true
    );
    assert!(artifact_uri.to_string().ends_with(".zasset"));
    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::PhysicsMaterial(physics_material));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata() {
    let root = unique_temp_project_root("artifact_store_shader_toml_metadata");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let mut editor_metadata = toml::Table::new();
    editor_metadata.insert(
        "inspector_group".to_string(),
        toml::Value::String("PBR".into()),
    );
    editor_metadata.insert(
        "generated_at".to_string(),
        toml::Value::Datetime(
            "2026-06-11T12:30:00Z"
                .parse::<toml::value::Datetime>()
                .unwrap(),
        ),
    );
    let mut texture_default = toml::Table::new();
    texture_default.insert(
        "fallback".to_string(),
        toml::Value::String("white".to_string()),
    );
    texture_default.insert("uv_channel".to_string(), toml::Value::Integer(1));
    let shader = ShaderAsset {
        uri: AssetUri::parse("res://shaders/pbr.zshader").unwrap(),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }"
            .to_string(),
        wgsl_source: String::new(),
        import_path: Some("shaders/pbr.wgsl".to_string()),
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: vec![
            RenderShaderDefinitionValue::uint("ALPHA_CLIP", 1),
            RenderShaderDefinitionValue::bool("USE_FOG", false),
        ],
        property_schema: vec![
            ShaderMaterialPropertyAsset {
                name: "tint".to_string(),
                kind: "vec4".to_string(),
                required: false,
                default: Some(toml::Value::Array(vec![
                    toml::Value::Float(1.0),
                    toml::Value::Float(0.8),
                    toml::Value::Float(0.6),
                    toml::Value::Float(1.0),
                ])),
                editor: BTreeMap::from([("widget".to_string(), "color".to_string())]),
            },
            ShaderMaterialPropertyAsset {
                name: "normal_map".to_string(),
                kind: "texture".to_string(),
                required: false,
                default: Some(toml::Value::Table(texture_default)),
                editor: BTreeMap::from([("slot".to_string(), "normal".to_string())]),
            },
        ],
        texture_slots: Vec::new(),
        editor: editor_metadata,
        pipeline_layout: Default::default(),
        validation_diagnostics: vec!["authoring note".to_string()],
    };
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Shader,
        AssetUri::parse("res://shaders/pbr.zshader").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(&paths, &metadata, &ImportedAsset::Shader(shader.clone()))
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert!(artifact_uri.to_string().contains("shaders/"));
    assert!(artifact_uri.to_string().ends_with(".zasset"));
    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::Shader(shader));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_roundtrips_animation_sequence_assets_in_binary_library() {
    let root = unique_temp_project_root("artifact_store_animation_sequence");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let sequence = sample_animation_sequence_asset();
    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::AnimationSequence,
        AssetUri::parse("res://animation/hero.sequence.zranim").unwrap(),
    );
    let store = ArtifactStore::default();

    let artifact_uri = store
        .write(
            &paths,
            &metadata,
            &ImportedAsset::AnimationSequence(sequence.clone()),
        )
        .unwrap();
    let loaded = store.read(&paths, &artifact_uri).unwrap();

    assert!(artifact_uri.to_string().contains("animation/sequences/"));
    assert!(artifact_uri.to_string().ends_with(".zasset"));
    assert_binary_artifact_payload(&paths, &artifact_uri);
    assert_eq!(loaded, ImportedAsset::AnimationSequence(sequence));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn artifact_store_rejects_text_library_artifacts() {
    let root = unique_temp_project_root("artifact_store_text_reject");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    let artifact_uri = AssetUri::parse("lib://materials/stale.json").unwrap();
    let artifact_path = paths.library_root().join(artifact_uri.path());
    fs::create_dir_all(artifact_path.parent().unwrap()).unwrap();
    fs::write(&artifact_path, br#"{"Material":{"name":"Stale"}}"#).unwrap();

    let error = ArtifactStore::default()
        .read(&paths, &artifact_uri)
        .unwrap_err();

    assert!(format!("{error:?}").contains("expected .zasset"));

    let _ = fs::remove_dir_all(root);
}

fn assert_binary_artifact_payload(paths: &ProjectPaths, artifact_uri: &AssetUri) {
    let payload = fs::read(paths.library_root().join(artifact_uri.path())).unwrap();
    assert!(payload.starts_with(b"ZRARTZ01"));
    assert_ne!(
        payload.get(b"ZRARTZ01".len()..b"ZRARTZ01".len() + 4),
        Some(&b"JSON"[..])
    );
    assert_ne!(
        payload.get(b"ZRARTZ01".len()..b"ZRARTZ01".len() + 4),
        Some(&b"BIN\0"[..])
    );
    let cache = zstd::stream::decode_all(&payload[b"ZRARTZ01".len()..]).unwrap();
    assert!(
        !matches!(cache.first(), Some(b'{') | Some(b'[')),
        "decompressed artifact cache should be bincode, not JSON text"
    );
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
