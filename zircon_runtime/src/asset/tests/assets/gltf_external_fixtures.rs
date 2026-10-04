//! 为外部 glTF 输入测试写出真实的伴随文件；导入器须从 .gltf 的相对 URI 解析 buffer 和图像。
//! 测试调用方只传主文件路径，避免用预先注入的字节掩盖旁路资源解析错误。

use std::fs;
use std::path::{Path, PathBuf};

/// 供外部资源正例使用：纹理经独立 PNG 和 buffer 导入，材质仍引用生成的 Texture 子资产。
pub(super) fn write_external_texture_gltf(root: &Path) -> PathBuf {
    let buffer_path = root.join("external_texture.bin");
    let image_path = root.join("external_albedo.png");
    let gltf_path = root.join("external_texture.gltf");

    fs::write(&image_path, tiny_png_rgba_bytes()).unwrap();

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
    { "uri": "external_texture.bin", "byteLength": 42 }
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
  "images": [
    { "uri": "external_albedo.png" }
  ],
  "textures": [
    { "source": 0 }
  ],
  "materials": [
    {
      "name": "ExternalTextureMaterial",
      "pbrMetallicRoughness": {
        "baseColorTexture": { "index": 0 }
      }
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1,
          "material": 0
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

/// 只写主文档而保留缺失的相对 buffer URI，供错误测试检查诊断包含原始文件名。
pub(super) fn write_missing_buffer_gltf(root: &Path) -> PathBuf {
    let gltf_path = root.join("missing_buffer.gltf");

    fs::write(
        &gltf_path,
        r#"
{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "missing.bin", "byteLength": 36 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 }
  ],
  "accessors": [
    {
      "bufferView": 0,
      "componentType": 5126,
      "count": 3,
      "type": "VEC3",
      "min": [0.0, 0.0, 0.0],
      "max": [1.0, 1.0, 0.0]
    }
  ],
  "meshes": [
    {
      "primitives": [
        { "attributes": { "POSITION": 0 } }
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

fn tiny_png_rgba_bytes() -> &'static [u8] {
    &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
        0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 255, 255, 255,
        127, 0, 9, 251, 3, 253, 42, 134, 227, 138, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ]
}
