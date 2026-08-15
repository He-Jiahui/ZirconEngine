use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::derive_animation_assets_from_model_source;
use zircon_runtime::asset::project::{
    AssetMetaDocument, ProjectManager, ProjectManifest, ProjectPaths,
};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::animation::{AnimationClipAsset, AnimationSkeletonAsset};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

#[test]
fn derive_animation_assets_from_model_source_writes_stable_sibling_skeleton_and_clip_files() {
    let root = unique_temp_dir("zircon_editor_derived_animation_assets");
    let assets_root = root.join("assets");
    let model_dir = assets_root.join("models");
    fs::create_dir_all(&model_dir).unwrap();
    let model_path = write_animated_gltf(&model_dir);

    let paths = ProjectPaths::from_root(&root).unwrap();
    let manifest = ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.save(paths.manifest_path()).unwrap();
    let project = ProjectManager::open(&root).unwrap();
    let first = derive_animation_assets_from_model_source(&project, &model_path).unwrap();
    let second = derive_animation_assets_from_model_source(&project, &model_path).unwrap();

    assert_eq!(first, second);
    assert_eq!(
        first.iter().map(ToString::to_string).collect::<Vec<_>>(),
        vec![
            "res://models/hero.idle.clip.zranim".to_string(),
            "res://models/hero.skeleton.zranim".to_string(),
        ]
    );

    let skeleton = AnimationSkeletonAsset::from_bytes(
        &fs::read(model_dir.join("hero.skeleton.zranim")).unwrap(),
    )
    .unwrap();
    assert_eq!(skeleton.name.as_deref(), Some("HeroRig"));
    assert_eq!(skeleton.bones.len(), 2);
    assert_eq!(skeleton.bones[0].name, "Root");
    assert_eq!(skeleton.bones[1].name, "Hand");
    assert_eq!(skeleton.bones[1].parent_index, Some(0));

    let clip =
        AnimationClipAsset::from_bytes(&fs::read(model_dir.join("hero.idle.clip.zranim")).unwrap())
            .unwrap();
    assert_eq!(clip.name.as_deref(), Some("Idle"));
    assert_eq!(
        clip.skeleton.locator.to_string(),
        "res://models/hero.skeleton.zranim"
    );
    assert_eq!(clip.tracks.len(), 2);
    assert!(clip.tracks.iter().any(|track| track.bone_name == "Root"));
    assert!(clip.tracks.iter().any(|track| track.bone_name == "Hand"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn derive_animation_assets_from_model_source_preserves_project_asset_ids_across_reimport_with_gltf_buffer_sidecars(
) {
    let root = unique_temp_dir("zircon_editor_derived_animation_reimport");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let model_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("models");
    fs::create_dir_all(&model_dir).unwrap();
    let model_path = write_animated_gltf(&model_dir);

    let mut manager = ProjectManager::open(&root).unwrap();
    let first_generated = derive_animation_assets_from_model_source(&manager, &model_path).unwrap();
    manager.scan_and_import().unwrap();

    let skeleton_uri = AssetUri::parse("res://models/hero.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://models/hero.idle.clip.zranim").unwrap();
    let first_skeleton_id = manager
        .registry()
        .get_by_locator(&skeleton_uri)
        .expect("derived skeleton should be imported")
        .id();
    let first_clip_id = manager
        .registry()
        .get_by_locator(&clip_uri)
        .expect("derived clip should be imported")
        .id();
    let first_skeleton_meta =
        AssetMetaDocument::load(model_dir.join("hero.skeleton.zranim.zmeta")).unwrap();
    let first_clip_meta =
        AssetMetaDocument::load(model_dir.join("hero.idle.clip.zranim.zmeta")).unwrap();

    let second_generated =
        derive_animation_assets_from_model_source(&manager, &model_path).unwrap();
    manager.scan_and_import().unwrap();

    let second_skeleton_id = manager
        .registry()
        .get_by_locator(&skeleton_uri)
        .expect("reimported skeleton should stay registered")
        .id();
    let second_clip_id = manager
        .registry()
        .get_by_locator(&clip_uri)
        .expect("reimported clip should stay registered")
        .id();
    let second_skeleton_meta =
        AssetMetaDocument::load(model_dir.join("hero.skeleton.zranim.zmeta")).unwrap();
    let second_clip_meta =
        AssetMetaDocument::load(model_dir.join("hero.idle.clip.zranim.zmeta")).unwrap();

    assert_eq!(first_generated, second_generated);
    assert_eq!(first_skeleton_id, second_skeleton_id);
    assert_eq!(first_clip_id, second_clip_id);
    assert_eq!(first_skeleton_meta.uuid, second_skeleton_meta.uuid);
    assert_eq!(first_clip_meta.uuid, second_clip_meta.uuid);
    assert!(
        !model_dir.join("hero.bin.meta.toml").exists(),
        "gltf buffer sidecars should not get runtime asset metadata sidecars"
    );
    assert!(
        !model_dir.join("hero.bin.zmeta").exists(),
        "gltf buffer sidecars should not get runtime asset metadata sidecars"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn animation_derivatives_stay_beside_a_model_in_the_second_manifest_root() {
    let root = unique_temp_dir("zircon_editor_second_root_animation_assets");
    let paths = ProjectPaths::from_root(&root).unwrap();
    let mut manifest = ProjectManifest::new(
        "Two Roots",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.asset_roots = vec![
        zircon_runtime_interface::project::RelPath::parse("game-assets").unwrap(),
        zircon_runtime_interface::project::RelPath::parse("shared-assets").unwrap(),
    ];
    manifest.save(paths.manifest_path()).unwrap();
    let shared_models = root.join("shared-assets/models");
    fs::create_dir_all(&shared_models).unwrap();
    let model_path = write_animated_gltf(&shared_models);
    let project = ProjectManager::open(&root).unwrap();

    derive_animation_assets_from_model_source(&project, &model_path).unwrap();

    assert!(shared_models.join("hero.skeleton.zranim").is_file());
    assert!(shared_models.join("hero.idle.clip.zranim").is_file());
    assert!(!root
        .join("game-assets/models/hero.skeleton.zranim")
        .exists());
    let _ = fs::remove_dir_all(root);
}

fn write_animated_gltf(model_dir: &Path) -> PathBuf {
    let model_path = model_dir.join("hero.gltf");
    let buffer_path = model_dir.join("hero.bin");

    let times = [0.0_f32, 1.0_f32];
    let root_translations = [[0.0_f32, 0.0, 0.0], [0.0_f32, 0.0, 0.0]];
    let hand_translations = [[0.2_f32, 0.8, 0.0], [0.4_f32, 1.1, 0.0]];

    let mut bytes = Vec::new();
    let times_offset = bytes.len();
    for value in times {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    let root_translation_offset = bytes.len();
    for sample in root_translations {
        for value in sample {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    let hand_translation_offset = bytes.len();
    for sample in hand_translations {
        for value in sample {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    fs::write(&buffer_path, bytes).unwrap();

    fs::write(
        &model_path,
        format!(
            r#"{{
  "asset": {{ "version": "2.0" }},
  "buffers": [
    {{ "uri": "hero.bin", "byteLength": {byte_length} }}
  ],
  "bufferViews": [
    {{ "buffer": 0, "byteOffset": {times_offset}, "byteLength": 8 }},
    {{ "buffer": 0, "byteOffset": {root_translation_offset}, "byteLength": 24 }},
    {{ "buffer": 0, "byteOffset": {hand_translation_offset}, "byteLength": 24 }}
  ],
  "accessors": [
    {{
      "bufferView": 0,
      "componentType": 5126,
      "count": 2,
      "type": "SCALAR",
      "min": [0.0],
      "max": [1.0]
    }},
    {{
      "bufferView": 1,
      "componentType": 5126,
      "count": 2,
      "type": "VEC3"
    }},
    {{
      "bufferView": 2,
      "componentType": 5126,
      "count": 2,
      "type": "VEC3"
    }}
  ],
  "nodes": [
    {{
      "name": "Root",
      "children": [1],
      "translation": [0.0, 0.0, 0.0]
    }},
    {{
      "name": "Hand",
      "translation": [0.2, 0.8, 0.0]
    }}
  ],
  "skins": [
    {{
      "name": "HeroRig",
      "joints": [0, 1],
      "skeleton": 0
    }}
  ],
  "animations": [
    {{
      "name": "Idle",
      "samplers": [
        {{ "input": 0, "output": 1, "interpolation": "LINEAR" }},
        {{ "input": 0, "output": 2, "interpolation": "LINEAR" }}
      ],
      "channels": [
        {{ "sampler": 0, "target": {{ "node": 0, "path": "translation" }} }},
        {{ "sampler": 1, "target": {{ "node": 1, "path": "translation" }} }}
      ]
    }}
  ],
  "scenes": [
    {{ "nodes": [0] }}
  ],
  "scene": 0
}}"#,
            byte_length = fs::metadata(&buffer_path).unwrap().len(),
            times_offset = times_offset,
            root_translation_offset = root_translation_offset,
            hand_translation_offset = hand_translation_offset,
        ),
    )
    .unwrap();

    model_path
}
