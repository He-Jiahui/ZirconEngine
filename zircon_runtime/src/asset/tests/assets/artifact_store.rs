use std::collections::BTreeMap;
use std::fs;

use crate::core::framework::render::{RenderShaderDefinitionValue, ShaderAssetKind};
use crate::core::framework::scene::physics::{
    PhysicsJointConstraintMetadata, PhysicsMassProperties, PhysicsMaterialMetadata,
};
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
    SceneMobilityAsset, SceneRigidBodyAsset, SceneRigidBodyTypeAsset, SceneScriptBindingAsset,
    ShaderAsset, ShaderImportRedirectAsset, ShaderMaterialPropertyAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset, TextureAsset, TransformAsset, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::render::RenderMeshTopology;

mod artifact_cache_assets;
mod binary_payloads;
mod material_data;
mod scene_components;
mod scene_script;

fn assert_binary_artifact_payload(paths: &ProjectPaths, artifact_uri: &AssetUri) {
    let payload = fs::read(paths.asset_artifact_root().join(artifact_uri.path())).unwrap();
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
