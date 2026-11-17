#![cfg(feature = "runtime")]

use std::path::PathBuf;

use zircon_plugin_gltf_importer_runtime::{asset_importer_descriptors, import_gltf};
use zircon_runtime::asset::{AssetImportContext, AssetUri, ImportedAsset, MESH_ATTRIBUTE_TANGENT};

// Same triangle and effective UV set as the existing geometry convergence
// case, with in-memory companion data so the production entry needs no files.
// Positions (0,+X,+Y), UV0 all zero, UV1 (0,+U,+V), indices (0,1,2).
const UV1_TRIANGLE: &[u8] = br#"{
  "asset": { "version": "2.0" },
  "extensionsUsed": ["KHR_texture_transform"],
  "buffers": [{ "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIA/AAAAAAAAAAAAAIA/AAABAAIA", "byteLength": 90 }],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 60, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 84, "byteLength": 6, "target": 34963 }
  ],
  "accessors": [
    { "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [1,1,0] },
    { "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC2" },
    { "bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2" },
    { "bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR" }
  ],
  "images": [{ "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg==" }],
  "textures": [{ "source": 0 }],
  "materials": [{ "normalTexture": {
    "index": 0, "texCoord": 0,
    "extensions": { "KHR_texture_transform": { "texCoord": 1 } }
  } }],
  "meshes": [{ "primitives": [{
    "attributes": { "POSITION": 0, "TEXCOORD_0": 1, "TEXCOORD_1": 2 },
    "indices": 3, "material": 0
  }] }],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0
}"#;

#[test]
fn stable_gltf_import_preserves_the_generated_effective_uv1_basis() {
    let uri = AssetUri::parse("res://meshes/normal-uv1.gltf").unwrap();
    let imported = import_gltf(&AssetImportContext::new(
        PathBuf::from("normal-uv1.gltf"),
        uri,
        UV1_TRIANGLE.to_vec(),
        toml::Table::new(),
    ))
    .unwrap();
    let mesh = imported
        .entries
        .iter()
        .find_map(|entry| match &entry.asset {
            ImportedAsset::Mesh(mesh) if entry.locator.label() == Some("Mesh0/Primitive0") => {
                Some(mesh)
            }
            _ => None,
        })
        .expect("the labelled mesh subasset owns the tangent stream");
    let tangents = mesh.attributes[MESH_ATTRIBUTE_TANGENT]
        .as_float32x4()
        .unwrap();
    assert_eq!(
        tangents,
        &[[1.0, 0.0, 0.0, 1.0]; 3],
        "effective UV1 must reconstruct the +V direction under the flat +Z normal"
    );
}

#[test]
fn stable_gltf_output_version_matches_the_manifest_projection() {
    let descriptors = asset_importer_descriptors();
    let [descriptor] = descriptors.as_slice() else {
        panic!("expected the one glTF import descriptor")
    };
    assert_eq!(descriptor.id, "gltf_importer.gltf");
    assert_eq!(
        descriptor.importer_version, 5,
        "generated tangent output changed"
    );
    let manifest: toml::Table = include_str!("../../plugin.toml").parse().unwrap();
    let row = &manifest["asset_importers"].as_array().unwrap()[0];
    assert_eq!(row["id"].as_str(), Some(descriptor.id.as_str()));
    assert_eq!(
        row["importer_version"].as_integer(),
        Some(i64::from(descriptor.importer_version))
    );
}
