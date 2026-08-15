use std::fs;
use std::path::{Path, PathBuf};

pub(in super::super) fn write_texture_transform_triangle_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("texture_transform_triangle.bin");
    let gltf_path = root.join("texture_transform_triangle.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        0.0_f32, 0.0, //
        0.5, 0.0, //
        0.0, 0.5,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        1.0_f32, 0.25, //
        0.25, 1.0, //
        0.75, 0.75,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"{
  "asset": { "version": "2.0" },
  "extensionsUsed": ["KHR_texture_transform"],
  "buffers": [
    { "uri": "texture_transform_triangle.bin", "byteLength": 90 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 60, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 84, "byteLength": 6, "target": 34963 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 1.0, 0.0]
    },
    {
      "bufferView": 1,
      "componentType": 5126,
      "count": 3,
      "type": "VEC2"
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 3,
      "type": "VEC2"
    },
    {
      "bufferView": 3,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    }
  ],
  "images": [
    {
      "uri": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4////fwAJ+wP9KobjigAAAABJRU5ErkJggg=="
    }
  ],
  "textures": [
    { "source": 0 }
  ],
  "materials": [
    {
      "name": "TextureTransformMaterial",
      "pbrMetallicRoughness": {
        "baseColorTexture": {
          "index": 0,
          "texCoord": 0,
          "extensions": {
            "KHR_texture_transform": {
              "offset": [0.1, 0.2],
              "scale": [0.3, 0.4],
              "texCoord": 1
            }
          }
        },
        "metallicRoughnessTexture": {
          "index": 0,
          "extensions": {
            "KHR_texture_transform": {
              "offset": [0.2, 0.3],
              "scale": [0.4, 0.5]
            }
          }
        }
      },
      "normalTexture": {
        "index": 0,
        "texCoord": 1,
        "extensions": {
          "KHR_texture_transform": {
            "offset": [0.3, 0.4],
            "scale": [0.5, 0.6],
            "texCoord": 0
          }
        }
      },
      "occlusionTexture": {
        "index": 0,
        "texCoord": 0,
        "strength": 0.25,
        "extensions": {
          "KHR_texture_transform": {
            "offset": [0.4, 0.5],
            "scale": [0.6, 0.7],
            "texCoord": 1
          }
        }
      },
      "emissiveTexture": {
        "index": 0,
        "texCoord": 1,
        "extensions": {
          "KHR_texture_transform": {
            "offset": [0.5, 0.6],
            "scale": [0.7, 0.8]
          }
        }
      }
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "TEXCOORD_0": 1,
            "TEXCOORD_1": 2
          },
          "indices": 3,
          "material": 0
        }
      ]
    }
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0
}"#,
    )
    .unwrap();

    gltf_path
}

pub(in super::super) fn write_two_primitive_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("two_primitives.bin");
    let gltf_path = root.join("two_primitives.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes.extend_from_slice(&[0, 0]);
    for value in [
        1.0_f32, 1.0, 0.0, //
        2.0, 1.0, 0.0, //
        1.0, 2.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"
{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "two_primitives.bin", "byteLength": 86 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963 },
    { "buffer": 0, "byteOffset": 44, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 80, "byteLength": 6, "target": 34963 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 1.0, 0.0]
    },
    {
      "bufferView": 1,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [1.0, 1.0, 0.0],
      "max": [2.0, 2.0, 0.0]
    },
    {
      "bufferView": 3,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    }
  ],
  "materials": [
    {
      "name": "FirstMaterial",
      "pbrMetallicRoughness": { "baseColorFactor": [1.0, 0.0, 0.0, 1.0] }
    },
    {
      "name": "SecondMaterial",
      "pbrMetallicRoughness": { "baseColorFactor": [0.0, 0.0, 1.0, 1.0] }
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1,
          "material": 0
        },
        {
          "attributes": { "POSITION": 2 },
          "indices": 3,
          "material": 1
        }
      ]
    }
  ],
  "nodes": [{ "mesh": 0 }],
  "scenes": [{ "nodes": [0] }],
  "scene": 0
}
"#,
    )
    .unwrap();

    gltf_path
}
