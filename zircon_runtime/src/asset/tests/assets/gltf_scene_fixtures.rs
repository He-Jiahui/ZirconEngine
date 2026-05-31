use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn write_two_scene_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("two_scenes.bin");
    let gltf_path = root.join("two_scenes.gltf");

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
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"
{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "two_scenes.bin", "byteLength": 42 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 6, "target": 34963 }
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
    }
  ],
  "materials": [
    {
      "name": "SharedSceneMaterial",
      "pbrMetallicRoughness": { "baseColorFactor": [0.4, 0.7, 0.2, 1.0] }
    }
  ],
  "meshes": [
    {
      "name": "SharedSceneMesh",
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1,
          "material": 0
        }
      ]
    }
  ],
  "nodes": [
    { "name": "FirstSceneNode", "mesh": 0 },
    { "name": "SecondSceneNode", "mesh": 0, "translation": [2.0, 0.0, 0.0] }
  ],
  "scenes": [
    { "name": "FirstScene", "nodes": [0] },
    { "name": "SecondScene", "nodes": [1] }
  ],
  "scene": 0
}
"#,
    )
    .unwrap();

    gltf_path
}
