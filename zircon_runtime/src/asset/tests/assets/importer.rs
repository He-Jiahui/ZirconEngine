use std::fs;
use std::path::Path;

use image::{ImageBuffer, ImageFormat, Rgb, Rgba};

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    sample_animation_sequence_asset, sample_physics_material_asset,
    write_default_animation_sequence, write_default_physics_material,
};
use crate::asset::{AssetImporter, AssetUri, ImportedAsset};

#[test]
fn importer_subtree_uses_ingest_namespace_without_service_shell() {
    let importer_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/asset/importer");
    let importer_mod = fs::read_to_string(importer_root.join("mod.rs")).unwrap_or_default();

    assert!(
        importer_mod.contains("mod ingest;"),
        "asset importer root should declare the ingest subtree directly"
    );
    assert!(
        !importer_mod.contains("mod service;"),
        "asset importer root should not keep a migration-smell service subtree"
    );
    assert!(
        importer_root.join("ingest").exists(),
        "asset importer ingest subtree should exist after the hard cutover"
    );
    assert!(
        !importer_root.join("service").exists(),
        "asset importer service subtree should be deleted after the hard cutover"
    );
}

#[test]
fn importer_decodes_png_and_jpeg_textures() {
    let root = unique_temp_project_root("texture_import");
    fs::create_dir_all(&root).unwrap();
    let png_path = root.join("checker.png");
    let jpg_path = root.join("checker.jpg");

    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 255])
        }
    })
    .save_with_format(&png_path, ImageFormat::Png)
    .unwrap();

    ImageBuffer::<Rgb<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgb([255, 0, 0])
        } else {
            Rgb([0, 0, 255])
        }
    })
    .save_with_format(&jpg_path, ImageFormat::Jpeg)
    .unwrap();

    let importer = AssetImporter::default();
    let png = importer
        .import_from_source(
            &png_path,
            &AssetUri::parse("res://textures/checker.png").unwrap(),
        )
        .unwrap();
    let jpg = importer
        .import_from_source(
            &jpg_path,
            &AssetUri::parse("res://textures/checker.jpg").unwrap(),
        )
        .unwrap();

    match png {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 2);
            assert_eq!(texture.height, 2);
            assert_eq!(texture.rgba.len(), 16);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    match jpg {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 2);
            assert_eq!(texture.height, 2);
            assert_eq!(texture.rgba.len(), 16);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_validates_wgsl_and_reports_errors() {
    let root = unique_temp_project_root("shader_import");
    fs::create_dir_all(&root).unwrap();
    let valid_path = root.join("pbr.wgsl");
    let invalid_path = root.join("broken.wgsl");
    fs::write(&valid_path, valid_wgsl()).unwrap();
    fs::write(&invalid_path, "@vertex fn vs_main( {").unwrap();

    let importer = AssetImporter::default();
    let valid = importer
        .import_from_source(
            &valid_path,
            &AssetUri::parse("res://shaders/pbr.wgsl").unwrap(),
        )
        .unwrap();
    let invalid = importer.import_from_source(
        &invalid_path,
        &AssetUri::parse("res://shaders/broken.wgsl").unwrap(),
    );

    match valid {
        ImportedAsset::Shader(shader) => {
            assert!(shader.source.contains("vs_main"));
            assert_eq!(shader.uri.to_string(), "res://shaders/pbr.wgsl");
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    assert!(invalid.is_err());
    assert!(invalid.unwrap_err().to_string().contains("wgsl"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_decodes_obj_and_gltf_into_model_assets() {
    let root = unique_temp_project_root("model_import");
    fs::create_dir_all(&root).unwrap();
    let obj_path = root.join("triangle.obj");
    fs::write(
        &obj_path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
",
    )
    .unwrap();

    let gltf_path = write_triangle_gltf(&root);
    let importer = AssetImporter::default();

    let obj = importer
        .import_from_source(
            &obj_path,
            &AssetUri::parse("res://models/triangle.obj").unwrap(),
        )
        .unwrap();
    let gltf = importer
        .import_from_source(
            &gltf_path,
            &AssetUri::parse("res://models/triangle.gltf").unwrap(),
        )
        .unwrap();

    match obj {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    match gltf {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_preserves_gltf_skinning_channels_on_model_vertices() {
    let root = unique_temp_project_root("skinned_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_skinned_triangle_gltf(&root);
    let importer = AssetImporter::default();

    let gltf = importer
        .import_from_source(
            &gltf_path,
            &AssetUri::parse("res://models/skinned_triangle.gltf").unwrap(),
        )
        .unwrap();

    match gltf {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert_eq!(model.primitives[0].vertices[0].joint_indices, [0, 1, 0, 0]);
            assert_eq!(model.primitives[0].vertices[1].joint_indices, [1, 0, 0, 0]);
            assert_eq!(
                model.primitives[0].vertices[0].joint_weights,
                [0.75, 0.25, 0.0, 0.0]
            );
            assert_eq!(
                model.primitives[0].vertices[1].joint_weights,
                [1.0, 0.0, 0.0, 0.0]
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_decodes_physics_material_and_animation_sequence_assets() {
    let root = unique_temp_project_root("physics_animation_import");
    fs::create_dir_all(&root).unwrap();
    let physics_material_path = root.join("default.physics_material.toml");
    let sequence_path = root.join("hero.sequence.zranim");

    write_default_physics_material(physics_material_path.clone());
    write_default_animation_sequence(sequence_path.clone());

    let importer = AssetImporter::default();
    let physics_material = importer
        .import_from_source(
            &physics_material_path,
            &AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap(),
        )
        .unwrap();
    let sequence = importer
        .import_from_source(
            &sequence_path,
            &AssetUri::parse("res://animation/hero.sequence.zranim").unwrap(),
        )
        .unwrap();

    assert_eq!(
        physics_material,
        ImportedAsset::PhysicsMaterial(sample_physics_material_asset())
    );
    assert_eq!(
        sequence,
        ImportedAsset::AnimationSequence(sample_animation_sequence_asset())
    );

    let _ = fs::remove_dir_all(root);
}

fn valid_wgsl() -> &'static str {
    r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.4, 0.2, 1.0);
}
"#
}

fn write_triangle_gltf(root: &Path) -> std::path::PathBuf {
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
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "triangle.bin", "byteLength": 42 }
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
  "meshes": [
    {
      "primitives": [
        {
          "attributes": { "POSITION": 0 },
          "indices": 1
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

fn write_skinned_triangle_gltf(root: &Path) -> std::path::PathBuf {
    let buffer_path = root.join("skinned_triangle.bin");
    let gltf_path = root.join("skinned_triangle.gltf");

    let mut bytes = Vec::new();
    for value in [
        0.0_f32, 0.0, 0.0, //
        1.0, 0.0, 0.0, //
        0.0, 1.0, 0.0,
    ] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    for joint in [
        0_u16, 1, 0, 0, //
        1, 0, 0, 0, //
        0, 0, 0, 0,
    ] {
        bytes.extend_from_slice(&joint.to_le_bytes());
    }
    for weight in [
        0.75_f32, 0.25, 0.0, 0.0, //
        1.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 0.0,
    ] {
        bytes.extend_from_slice(&weight.to_le_bytes());
    }
    for index in [0_u16, 1, 2] {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &gltf_path,
        r#"{
  "asset": { "version": "2.0" },
  "buffers": [
    { "uri": "skinned_triangle.bin", "byteLength": 114 }
  ],
  "bufferViews": [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36, "target": 34962 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 24, "target": 34962 },
    { "buffer": 0, "byteOffset": 60, "byteLength": 48, "target": 34962 },
    { "buffer": 0, "byteOffset": 108, "byteLength": 6, "target": 34963 }
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
      "type": "VEC4"
    },
    {
      "bufferView": 2,
      "componentType": 5126,
      "count": 3,
      "type": "VEC4"
    },
    {
      "bufferView": 3,
      "componentType": 5123,
      "count": 3,
      "type": "SCALAR"
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "JOINTS_0": 1,
            "WEIGHTS_0": 2
          },
          "indices": 3
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
