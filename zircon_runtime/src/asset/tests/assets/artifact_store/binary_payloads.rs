use super::*;
use crate::core::framework::render::{
    RenderImageColorSpace, TextureMipFilter, TextureMipPolicy, TextureUsageHint,
};

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
        lods: vec![crate::asset::SceneMeshLodLevelAsset {
            min_distance: 16.0,
            model: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Mesh0"),
            mesh: None,
            material: asset_reference("res://models/kenney_graveyard/fence-gate.glb#Material0"),
            primitives: Vec::new(),
        }],
    };

    let bytes = bincode::serialize(&mesh).unwrap();
    let loaded = bincode::deserialize::<crate::asset::SceneMeshInstanceAsset>(&bytes).unwrap();

    assert_eq!(loaded, mesh);
}

#[test]
fn artifact_store_roundtrips_mesh_assets_with_binary_attribute_payloads() {
    let root = unique_temp_project_root("artifact_store_mesh_binary_payloads");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

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
        mesh_sdf: None,
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
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

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
fn artifact_store_rejects_runtime_mip_assets_with_unsupported_storage_format() {
    let root = unique_temp_project_root("artifact_store_runtime_mip_format_validation");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let mut texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/stale_runtime_mips.exr").unwrap(),
        2,
        2,
        vec![0; 16],
    );
    let mut descriptor = texture.texture_descriptor();
    descriptor.format = "rgba16float".to_string();
    descriptor.color_space = RenderImageColorSpace::Linear;
    descriptor.metadata.color_space = RenderImageColorSpace::Linear;
    descriptor.metadata.usage_hint = TextureUsageHint::Hdr;
    descriptor.metadata.mip_policy = TextureMipPolicy::GenerateRuntime;
    descriptor.metadata.mip_filter = TextureMipFilter::Box;
    descriptor.mip_count = 2;
    texture = texture.with_descriptor(descriptor);

    let metadata = ResourceRecord::new(
        AssetId::new(),
        AssetKind::Texture,
        AssetUri::parse("res://textures/stale_runtime_mips.exr").unwrap(),
    );
    let store = ArtifactStore::default();
    let error = store
        .write(&paths, &metadata, &ImportedAsset::Texture(texture))
        .unwrap_err();

    assert!(matches!(
        error,
        AssetImportError::Parse(message)
            if message.contains("runtime mip generation supports only rgba8unorm storage")
    ));

    let _ = fs::remove_dir_all(root);
}
