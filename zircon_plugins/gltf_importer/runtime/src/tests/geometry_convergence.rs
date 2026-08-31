use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{
    AssetImportContext, AssetUri, ImportedAsset, MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_TANGENT,
};

use super::import_gltf;

#[test]
fn stable_importer_preserves_authored_tangent_and_color_in_mesh_owner() {
    let root = unique_temp_root("gltf_geometry_authored_channels");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_authored_tangent_color_gltf(&root);
    let root_uri = AssetUri::parse("res://models/authored_channels.gltf").unwrap();
    let outcome = import_path(gltf_path, root_uri.clone());

    match &outcome.root_entry().unwrap().asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert!(model.primitives[0].vertices.is_empty());
            assert!(model.primitives[0].indices.is_empty());
            assert!(model.primitives[0].virtual_geometry.is_none());
        }
        other => panic!("unexpected root asset: {other:?}"),
    }

    let mesh = imported_mesh(&outcome, &root_uri);
    assert_eq!(
        mesh.attributes[MESH_ATTRIBUTE_TANGENT]
            .as_float32x4()
            .unwrap(),
        [
            [1.0, 0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0, -1.0],
        ]
    );
    assert_eq!(
        mesh.attributes[MESH_ATTRIBUTE_COLOR]
            .as_float32x4()
            .unwrap(),
        [
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ]
    );
    assert!(
        mesh.virtual_geometry.is_none(),
        "default import settings must not eagerly cook optional virtual geometry"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn stable_importer_uses_normal_texture_effective_uv_for_mikktspace() {
    let root = unique_temp_root("gltf_geometry_mikktspace_uv1");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_normal_mapped_uv1_gltf(&root);
    let root_uri = AssetUri::parse("res://models/normal_uv1.gltf").unwrap();
    let outcome = import_path(gltf_path, root_uri.clone());

    let mesh = imported_mesh(&outcome, &root_uri);
    let tangents = mesh.attributes[MESH_ATTRIBUTE_TANGENT]
        .as_float32x4()
        .expect("normal-mapped glTF mesh must own generated float32x4 tangents");
    assert_eq!(tangents.len(), 3);
    for tangent in tangents {
        let tangent_length_squared =
            tangent[0] * tangent[0] + tangent[1] * tangent[1] + tangent[2] * tangent[2];
        assert!(tangent_length_squared > 0.9);
        assert!(tangent.iter().all(|component| component.is_finite()));
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn stable_importer_rejects_clearcoat_only_normal_without_authored_tangent_space() {
    let root = unique_temp_root("gltf_geometry_clearcoat_only_missing_tangent");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_clearcoat_only_missing_tangent_gltf(&root);
    let source_bytes = fs::read(&gltf_path).unwrap();

    let error = import_gltf(&AssetImportContext::new(
        gltf_path,
        AssetUri::parse("res://models/clearcoat_only_missing_tangent.gltf").unwrap(),
        source_bytes,
        toml::Table::new(),
    ))
    .expect_err("clearcoat-only normal mapping needs an authored tangent space");

    let message = error.to_string();
    assert!(message.contains("clearcoat normal texture"));
    assert!(message.contains("authored NORMAL and TANGENT"));

    let _ = fs::remove_dir_all(root);
}

fn import_path(path: PathBuf, root_uri: AssetUri) -> zircon_runtime::asset::AssetImportOutcome {
    let source_bytes = fs::read(&path).unwrap();
    import_gltf(&AssetImportContext::new(
        path,
        root_uri,
        source_bytes,
        toml::Table::new(),
    ))
    .unwrap()
}

fn imported_mesh<'a>(
    outcome: &'a zircon_runtime::asset::AssetImportOutcome,
    root_uri: &AssetUri,
) -> &'a zircon_runtime::asset::MeshAsset {
    let mesh_uri = crate::subassets::gltf_label_reference(root_uri, "Mesh0/Primitive0").locator;
    let entry = outcome
        .entries
        .iter()
        .find(|entry| entry.uri == mesh_uri)
        .expect("Mesh0/Primitive0 entry");
    match &entry.asset {
        ImportedAsset::Mesh(mesh) => mesh,
        other => panic!("unexpected Mesh0/Primitive0 asset: {other:?}"),
    }
}

fn write_authored_tangent_color_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("authored_channels.bin");
    let gltf_path = root.join("authored_channels.gltf");
    let mut bytes = Vec::new();
    push_f32s(&mut bytes, &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    push_f32s(&mut bytes, &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
    push_f32s(
        &mut bytes,
        &[
            1.0, 0.0, 0.0, -1.0, 1.0, 0.0, 0.0, -1.0, 1.0, 0.0, 0.0, -1.0,
        ],
    );
    push_f32s(
        &mut bytes,
        &[1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0],
    );
    push_f32s(&mut bytes, &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
    push_u16s(&mut bytes, &[0, 1, 2]);
    fs::write(&buffer_path, &bytes).unwrap();
    fs::write(
        &gltf_path,
        format!(
            r#"{{
  "asset": {{ "version": "2.0" }},
  "buffers": [{{ "uri": "authored_channels.bin", "byteLength": {} }}],
  "bufferViews": [
    {{ "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 36, "byteLength": 36, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 72, "byteLength": 48, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 120, "byteLength": 48, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 168, "byteLength": 24, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 192, "byteLength": 6, "target": 34963 }}
  ],
  "accessors": [
    {{ "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [1,1,0] }},
    {{ "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3" }},
    {{ "bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC4" }},
    {{ "bufferView": 3, "componentType": 5126, "count": 3, "type": "VEC4" }},
    {{ "bufferView": 4, "componentType": 5126, "count": 3, "type": "VEC2" }},
    {{ "bufferView": 5, "componentType": 5123, "count": 3, "type": "SCALAR" }}
  ],
  "meshes": [{{ "primitives": [{{
    "attributes": {{ "POSITION": 0, "NORMAL": 1, "TANGENT": 2, "COLOR_0": 3, "TEXCOORD_0": 4 }},
    "indices": 5
  }}] }}],
  "nodes": [{{ "mesh": 0 }}],
  "scenes": [{{ "nodes": [0] }}],
  "scene": 0
}}"#,
            bytes.len()
        ),
    )
    .unwrap();
    gltf_path
}

fn write_normal_mapped_uv1_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("normal_uv1.bin");
    let gltf_path = root.join("normal_uv1.gltf");
    let mut bytes = Vec::new();
    push_f32s(&mut bytes, &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    push_f32s(&mut bytes, &[0.0; 6]);
    push_f32s(&mut bytes, &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
    push_u16s(&mut bytes, &[0, 1, 2]);
    fs::write(&buffer_path, &bytes).unwrap();
    fs::write(
        &gltf_path,
        format!(
            r#"{{
  "asset": {{ "version": "2.0" }},
  "extensionsUsed": ["KHR_texture_transform"],
  "buffers": [{{ "uri": "normal_uv1.bin", "byteLength": {} }}],
  "bufferViews": [
    {{ "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 36, "byteLength": 24, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 60, "byteLength": 24, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 84, "byteLength": 6, "target": 34963 }}
  ],
  "accessors": [
    {{ "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [1,1,0] }},
    {{ "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC2" }},
    {{ "bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2" }},
    {{ "bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR" }}
  ],
  "images": [{{ "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg==" }}],
  "textures": [{{ "source": 0 }}],
  "materials": [{{ "normalTexture": {{
    "index": 0,
    "texCoord": 0,
    "extensions": {{ "KHR_texture_transform": {{ "texCoord": 1 }} }}
  }} }}],
  "meshes": [{{ "primitives": [{{
    "attributes": {{ "POSITION": 0, "TEXCOORD_0": 1, "TEXCOORD_1": 2 }},
    "indices": 3,
    "material": 0
  }}] }}],
  "nodes": [{{ "mesh": 0 }}],
  "scenes": [{{ "nodes": [0] }}],
  "scene": 0
}}"#,
            bytes.len()
        ),
    )
    .unwrap();
    gltf_path
}

fn write_clearcoat_only_missing_tangent_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("clearcoat_only_missing_tangent.bin");
    let gltf_path = root.join("clearcoat_only_missing_tangent.gltf");
    let mut bytes = Vec::new();
    push_f32s(&mut bytes, &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    push_f32s(&mut bytes, &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
    push_f32s(&mut bytes, &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
    push_u16s(&mut bytes, &[0, 1, 2]);
    fs::write(&buffer_path, &bytes).unwrap();
    fs::write(
        &gltf_path,
        format!(
            r#"{{
  "asset": {{ "version": "2.0" }},
  "extensionsUsed": ["KHR_materials_clearcoat"],
  "buffers": [{{ "uri": "clearcoat_only_missing_tangent.bin", "byteLength": {} }}],
  "bufferViews": [
    {{ "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 36, "byteLength": 36, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 72, "byteLength": 24, "target": 34962 }},
    {{ "buffer": 0, "byteOffset": 96, "byteLength": 6, "target": 34963 }}
  ],
  "accessors": [
    {{ "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [1,1,0] }},
    {{ "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3" }},
    {{ "bufferView": 2, "componentType": 5126, "count": 3, "type": "VEC2" }},
    {{ "bufferView": 3, "componentType": 5123, "count": 3, "type": "SCALAR" }}
  ],
  "images": [{{ "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg==" }}],
  "textures": [{{ "source": 0 }}],
  "materials": [{{
    "extensions": {{
      "KHR_materials_clearcoat": {{
        "clearcoatFactor": 1.0,
        "clearcoatNormalTexture": {{ "index": 0, "texCoord": 0 }}
      }}
    }}
  }}],
  "meshes": [{{ "primitives": [{{
    "attributes": {{ "POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2 }},
    "indices": 3,
    "material": 0
  }}] }}],
  "nodes": [{{ "mesh": 0 }}],
  "scenes": [{{ "nodes": [0] }}],
  "scene": 0
}}"#,
            bytes.len()
        ),
    )
    .unwrap();
    gltf_path
}

fn push_f32s(bytes: &mut Vec<u8>, values: &[f32]) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn push_u16s(bytes: &mut Vec<u8>, values: &[u16]) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn unique_temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let artifact_root = std::env::var_os("ZIRCON_TEST_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .expect("test working directory")
                .join("target/zircon-test-artifacts")
        });
    artifact_root.join(format!("zircon_{label}_{nanos}"))
}
