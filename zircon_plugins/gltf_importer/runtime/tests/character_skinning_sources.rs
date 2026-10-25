#![cfg(feature = "runtime")]

//! Exercises checked-in character data through the production CPU importer and sampler.
//! These checks do not establish preview-session, GPU frame, picking, or editor readiness.

use std::path::Path;

use zircon_plugin_gltf_importer_runtime::import_gltf;
use zircon_runtime::animation::DefaultAnimationManager;
use zircon_runtime::asset::{
    AssetImportContext, AssetUri, ImportedAsset, MESH_ATTRIBUTE_JOINT_INDEX,
    MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_POSITION,
};
use zircon_runtime::core::framework::animation::{
    AnimationClipAsset, AnimationManager, AnimationPoseOutput, AnimationSkeletonAsset,
};

#[test]
fn character_wolf_decodes_skin_and_samples_real_poses() {
    validate_character("creatures/wolf_basic.glb", 1, 48, 14);
}

#[test]
fn character_knight_decodes_skins_and_samples_real_poses() {
    validate_character("chars/players/knight.glb", 9, 23, 22);
}

fn validate_character(
    relative_path: &str,
    skin_count: usize,
    bone_count: usize,
    clip_count: usize,
) {
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/woc/assets/m8/models")
        .join(relative_path);
    let bytes = std::fs::read(&source_path).expect("checked-in character GLB must be present");
    // Inspect source joint ordinals without a second library extension policy;
    // the production importer below owns required-extension validation and decoding.
    let source = gltf::Gltf::from_slice_without_validation(&bytes)
        .expect("checked-in character GLB structure must parse");
    assert_eq!(source.document.skins().count(), skin_count);
    assert_eq!(source.document.animations().count(), clip_count);
    assert!(source
        .document
        .views()
        .any(|view| { view.extension_value("EXT_meshopt_compression").is_some() }));

    let outcome = import_gltf(&AssetImportContext::new(
        source_path,
        AssetUri::parse(&format!("res://characters/{relative_path}")).unwrap(),
        bytes.clone(),
        toml::Table::new(),
    ))
    .unwrap_or_else(|error| panic!("{relative_path}: actual character import failed: {error}"));
    assert!(matches!(
        outcome.root_entry().map(|entry| &entry.asset),
        Some(ImportedAsset::Model(_))
    ));

    for skin in source.document.skins() {
        let label = format!("Skin{}/Skeleton", skin.index());
        let skeleton = outcome
            .entries
            .iter()
            .find_map(|entry| match &entry.asset {
                ImportedAsset::AnimationSkeleton(skeleton)
                    if entry.locator.label() == Some(label.as_str()) =>
                {
                    Some(skeleton)
                }
                _ => None,
            })
            .expect("each source skin must produce its typed skeleton subasset");
        assert_eq!(skeleton.bones.len(), bone_count, "{relative_path}/{label}");
        // Joint ordinals are the mesh influence and pose slots; source node order may differ.
        for (bone, joint) in skeleton.bones.iter().zip(skin.joints()) {
            let expected_name = format!(
                "Node{}:{}",
                joint.index(),
                joint.name().expect("checked-in character joints are named")
            );
            assert_eq!(bone.name, expected_name);
            let (translation, rotation, scale) = joint.transform().decomposed();
            assert_eq!(bone.local_translation, translation);
            assert_eq!(bone.local_rotation, rotation);
            assert_eq!(bone.local_scale, scale);
        }
        let encoded = skeleton
            .to_bytes()
            .expect("skeleton hierarchy must be valid");
        assert_eq!(
            AnimationSkeletonAsset::from_bytes(&encoded).unwrap(),
            *skeleton,
            "skeleton persistence must retain hierarchy and joint slots"
        );
    }

    let mut decoded_meshes = 0;
    for entry in &outcome.entries {
        let ImportedAsset::Mesh(mesh) = &entry.asset else {
            continue;
        };
        decoded_meshes += 1;
        let vertex_count = mesh.vertex_count().expect("mesh channels must agree");
        assert!(vertex_count > 0);
        let positions = mesh.attributes[MESH_ATTRIBUTE_POSITION]
            .as_float32x3()
            .expect("positions must be decoded");
        assert!(positions.iter().flatten().all(|value| value.is_finite()));
        let joints = mesh.attributes[MESH_ATTRIBUTE_JOINT_INDEX]
            .as_uint16x4()
            .expect("joint slots must be decoded");
        let weights = mesh.attributes[MESH_ATTRIBUTE_JOINT_WEIGHT]
            .as_float32x4()
            .expect("skin weights must be decoded");
        assert_eq!(joints.len(), vertex_count);
        assert_eq!(weights.len(), vertex_count);
        for (indices, weights) in joints.iter().zip(weights) {
            assert!(weights
                .iter()
                .all(|weight| weight.is_finite() && *weight >= 0.0));
            assert!((weights.iter().sum::<f32>() - 1.0).abs() < 1.0e-3);
            for (index, weight) in indices.iter().zip(weights) {
                if *weight > 0.0 {
                    assert!(usize::from(*index) < bone_count);
                }
            }
        }
        let matrices = &mesh
            .skin
            .as_ref()
            .expect("mesh must retain its skin")
            .inverse_bind_matrices;
        assert_eq!(matrices.len(), bone_count);
        assert!(matrices
            .iter()
            .flatten()
            .flatten()
            .all(|value| value.is_finite()));
    }
    assert_eq!(decoded_meshes, skin_count);

    let manager = DefaultAnimationManager::default();
    let mut sampled_clips = 0;
    let mut changing_clips = 0;
    for entry in &outcome.entries {
        let ImportedAsset::AnimationClip(clip) = &entry.asset else {
            continue;
        };
        sampled_clips += 1;
        assert!(clip.duration_seconds.is_finite() && clip.duration_seconds > 0.0);
        assert!(!clip.tracks.is_empty());
        let skeleton = outcome
            .entries
            .iter()
            .find_map(|entry| match &entry.asset {
                ImportedAsset::AnimationSkeleton(skeleton)
                    if entry.locator == clip.skeleton.locator =>
                {
                    Some(skeleton)
                }
                _ => None,
            })
            .expect("clip must reference an imported typed skeleton");
        let encoded = clip.to_bytes().expect("clip must serialize");
        assert_eq!(AnimationClipAsset::from_bytes(&encoded).unwrap(), *clip);
        let initial = manager
            .sample_clip_pose(skeleton, clip, 0.0, false)
            .unwrap();
        validate_pose(&initial, skeleton);
        let mut changed = false;
        for fraction in [0.25, 0.5, 0.75, 1.0] {
            let pose = manager
                .sample_clip_pose(skeleton, clip, clip.duration_seconds * fraction, false)
                .unwrap();
            validate_pose(&pose, skeleton);
            changed |= pose
                .bones
                .iter()
                .zip(&initial.bones)
                .any(|(bone, initial)| {
                    let current = &bone.local_transform;
                    let initial = &initial.local_transform;
                    current.translation.distance_squared(initial.translation) > 1.0e-8
                        || current.rotation.dot(initial.rotation).abs() < 1.0 - 1.0e-6
                        || current.scale.distance_squared(initial.scale) > 1.0e-8
                });
        }
        changing_clips += usize::from(changed);
    }
    assert_eq!(sampled_clips, clip_count);
    assert!(
        changing_clips > 0,
        "real clips must change the sampled character pose"
    );
}

fn validate_pose(pose: &AnimationPoseOutput, skeleton: &AnimationSkeletonAsset) {
    assert_eq!(pose.bones.len(), skeleton.bones.len());
    for (pose, source) in pose.bones.iter().zip(&skeleton.bones) {
        assert_eq!(pose.name, source.name);
        let transform = &pose.local_transform;
        assert!(transform.translation.is_finite());
        assert!(transform.rotation.is_finite());
        assert!((transform.rotation.length_squared() - 1.0).abs() < 1.0e-4);
        assert!(transform.scale.is_finite());
    }
}
