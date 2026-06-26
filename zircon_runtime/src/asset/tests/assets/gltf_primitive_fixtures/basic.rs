use std::fs;
use std::path::{Path, PathBuf};

pub(in super::super) fn write_triangle_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("triangle.bin");
    let gltf_path = root.join("triangle.gltf");

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
        1.0_f32, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&0.0_f32.to_le_bytes());
    for value in [0.0_f32, 0.0, 0.0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for value in [0.1_f32, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.1] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "triangle.bin", "byteLength": 160 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963 },
    { "buffer": 0, "byteOffset": 44, "byteLength": 64 },
    { "buffer": 0, "byteOffset": 108, "byteLength": 4 },
    { "buffer": 0, "byteOffset": 112, "byteLength": 12 },
    { "buffer": 0, "byteOffset": 124, "byteLength": 36, "target": 34962 }
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
      "count": 1,
      "type": "MAT4"
    },
    {
      "bufferView": 3,
      "componentType": 5126,
      "count": 1,
      "type": "SCALAR",
      "min": [0.0],
      "max": [0.0]
    },
    {
      "bufferView": 4,
      "componentType": 5126,
      "count": 1,
      "type": "VEC3"
    },
    {
      "bufferView": 5,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3"
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
      "name": "TriangleMaterial",
      "pbrMetallicRoughness": {
        "baseColorFactor": [0.2, 0.3, 0.4, 1.0],
        "baseColorTexture": { "index": 0 },
        "metallicFactor": 0.5,
        "roughnessFactor": 0.6
      }
    }
  ],
  "meshes": [
    {
      "name": "TriangleMesh",
      "weights": [0.5],
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1,
          "material": 0,
          "targets": [{ "POSITION": 5 }]
        }
      ]
    }
  ],
  "nodes": [{ "name": "TriangleNode", "mesh": 0, "skin": 0 }],
  "skins": [{ "inverseBindMatrices": 2, "joints": [0] }],
  "animations": [
    {
      "samplers": [{ "input": 3, "output": 4, "interpolation": "LINEAR" }],
      "channels": [{ "sampler": 0, "target": { "node": 0, "path": "translation" } }]
    }
  ],
  "scenes": [{ "name": "SceneRoot", "nodes": [0] }],
  "scene": 0
}"#,
    )
    .unwrap();

    gltf_path
}

pub(in super::super) fn write_line_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("line.bin");
    let gltf_path = root.join("line.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"
{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "line.bin", "byteLength": 24 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 24, "target": 34962 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 2,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 0.0, 0.0]
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "mode": 1,
          "attributes": { "POSITION": 0 }
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
