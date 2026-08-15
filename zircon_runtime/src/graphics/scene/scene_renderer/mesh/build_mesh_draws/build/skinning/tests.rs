use std::collections::BTreeMap;

use super::{
    prepare_skinned_mesh_asset_primitive, prepare_skinned_model_primitive,
    skin_mesh_asset_primitive, skin_model_primitive, SkinnedMeshJointPalette,
};
use crate::asset::{
    AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset, MeshVertex,
    ModelPrimitiveAsset, MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_JOINT_INDEX,
    MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION,
    MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use crate::core::framework::animation::{AnimationSkeletonAsset, AnimationSkeletonBoneAsset};
use crate::core::framework::render::RenderMeshTopology;
use crate::core::math::{Quat, Transform, Vec2, Vec3};
use crate::graphics::scene::scene_renderer::mesh::skinning::SKINNED_MESH_MAX_JOINT_MATRICES;

#[test]
fn joint_palette_composes_pose_world_against_bind_world_matrices() {
    let skeleton = unit_test_skeleton();
    let pose = joint_quarter_turn_pose();

    let palette = SkinnedMeshJointPalette::from_skeleton_pose(&skeleton, &pose)
        .expect("expected unit skeleton pose to produce a joint palette");

    assert_eq!(palette.matrices().len(), 2);
    assert!(
        palette.matrices()[0]
            .transform_point3(Vec3::ZERO)
            .abs_diff_eq(Vec3::ZERO, 1.0e-4),
        "expected root joint matrix to preserve the bind-space origin"
    );
    assert!(
        palette.matrices()[1]
            .transform_point3(Vec3::new(2.0, 0.0, 0.0))
            .abs_diff_eq(Vec3::new(1.0, 1.0, 0.0), 1.0e-4),
        "expected joint palette matrix to match the CPU skinning fallback"
    );
}

#[test]
fn joint_palette_reports_missing_parent_bone_reference() {
    let mut skeleton = unit_test_skeleton();
    skeleton.bones[1].parent_index = Some(99);
    let pose = joint_quarter_turn_pose();

    let error = SkinnedMeshJointPalette::from_skeleton_pose(&skeleton, &pose)
        .expect_err("expected invalid parent index to reject the joint palette");

    assert!(
        error.contains("references missing parent 99"),
        "expected missing-parent error, got {error}"
    );
}

#[test]
fn joint_palette_storage_packs_gpu_matrices_and_count() {
    let skeleton = unit_test_skeleton();
    let pose = joint_quarter_turn_pose();
    let palette = SkinnedMeshJointPalette::from_skeleton_pose(&skeleton, &pose)
        .expect("expected unit skeleton pose to produce a joint palette");

    let storage = palette
        .to_storage()
        .expect("expected small palette to fit the storage GPU ABI");

    assert_eq!(storage.joint_count(), 2);
    assert_eq!(
        storage.joint_matrices()[1],
        palette.matrices()[1].to_cols_array_2d()
    );
    assert_eq!(
        storage.joint_matrices()[2],
        crate::core::math::Mat4::IDENTITY.to_cols_array_2d(),
        "unused joint slots stay identity so accidental zero-weight reads are neutral"
    );
}

#[test]
fn joint_palette_storage_rejects_current_storage_limit_overflow() {
    let palette = SkinnedMeshJointPalette {
        joint_matrices: vec![
            crate::core::math::Mat4::IDENTITY;
            SKINNED_MESH_MAX_JOINT_MATRICES + 1
        ],
    };

    let error = palette
        .to_storage()
        .expect_err("expected oversized palette to reject the fixed storage ABI");

    assert!(
        error.contains("supports at most 256"),
        "expected storage-limit error, got {error}"
    );
}

#[test]
fn prepared_skinned_model_primitive_keeps_cpu_skinning_when_palette_exceeds_storage_limit() {
    let primitive = ModelPrimitiveAsset {
        vertices: vec![MeshVertex::new(Vec3::ZERO, Vec3::X, Vec2::ZERO)],
        indices: vec![0],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };
    let skeleton = oversized_storage_skeleton();
    let pose = AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones: Vec::new(),
    };

    let prepared = prepare_skinned_model_primitive(&primitive, &skeleton, &pose)
        .expect("expected oversized storage palette to keep CPU fallback primitive");

    assert_eq!(prepared.primitive.indices, vec![0]);
    assert_eq!(
        prepared.primitive.vertices[0].position,
        Vec3::ZERO.to_array()
    );
    assert!(
        prepared.joint_palette_storage.is_none(),
        "oversized storage palettes should not drop the CPU-skinned draw"
    );
}

#[test]
fn skin_model_primitive_rotates_weighted_vertex_around_joint_bind_origin() {
    let primitive = ModelPrimitiveAsset {
        vertices: vec![
            MeshVertex::new(Vec3::new(2.0, 0.0, 0.0), Vec3::X, Vec2::ZERO)
                .with_tangent([1.0, 0.0, 0.0, -1.0])
                .with_color([0.25, 0.5, 0.75, 1.0])
                .with_skinning([1, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]),
        ],
        indices: vec![0],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };
    let skeleton = unit_test_skeleton();
    let pose = joint_quarter_turn_pose();

    let skinned = skin_model_primitive(&primitive, &skeleton, &pose)
        .expect("expected CPU skinning helper to skin a valid weighted primitive");
    let vertex = &skinned.vertices[0];

    assert!(
        Vec3::from_array(vertex.position).abs_diff_eq(Vec3::new(1.0, 1.0, 0.0), 1.0e-4),
        "expected joint-space rotation around the bind-space joint origin to move the vertex"
    );
    assert!(
        Vec3::from_array(vertex.normal).abs_diff_eq(Vec3::Y, 1.0e-4),
        "expected skinned normal to follow the posed joint rotation"
    );
    assert!(
        Vec3::from_array([vertex.tangent[0], vertex.tangent[1], vertex.tangent[2]])
            .abs_diff_eq(Vec3::Y, 1.0e-4),
        "expected skinned tangent to follow the posed joint rotation"
    );
    assert_eq!(vertex.tangent[3], -1.0);
    assert_eq!(vertex.color, [0.25, 0.5, 0.75, 1.0]);
}

#[test]
fn skin_mesh_asset_primitive_converts_direct_mesh_attributes_before_skinning() {
    let mesh = MeshAsset::new(
        AssetUri::parse("res://meshes/skinned-direct.zmesh").unwrap(),
        RenderMeshTopology::PointList,
        BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[2.0, 0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[1.0, 0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_TANGENT.to_string(),
                MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, -1.0]]),
            ),
            (
                MESH_ATTRIBUTE_COLOR.to_string(),
                MeshAttributeValues::Float32x4(vec![[0.25, 0.5, 0.75, 1.0]]),
            ),
            (
                MESH_ATTRIBUTE_JOINT_INDEX.to_string(),
                MeshAttributeValues::Uint16x4(vec![[1, 0, 0, 0]]),
            ),
            (
                MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
                MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, 0.0]]),
            ),
        ]),
        Some(MeshIndices::U32(vec![0])),
    )
    .unwrap();
    let skeleton = unit_test_skeleton();
    let pose = joint_quarter_turn_pose();

    let skinned = skin_mesh_asset_primitive(&mesh, &skeleton, &pose, &[])
        .expect("expected direct mesh payload to skin through the shared primitive helper");
    let vertex = &skinned.vertices[0];

    assert_eq!(skinned.indices, vec![0]);
    assert!(skinned.mesh.is_none());
    assert!(
        Vec3::from_array(vertex.position).abs_diff_eq(Vec3::new(1.0, 1.0, 0.0), 1.0e-4),
        "expected direct mesh joint attributes to drive the same skinned position"
    );
    assert!(
        Vec3::from_array(vertex.normal).abs_diff_eq(Vec3::Y, 1.0e-4),
        "expected direct mesh normal to follow the posed joint rotation"
    );
    assert!(
        Vec3::from_array([vertex.tangent[0], vertex.tangent[1], vertex.tangent[2]])
            .abs_diff_eq(Vec3::Y, 1.0e-4),
        "expected direct mesh tangent to follow the posed joint rotation"
    );
    assert_eq!(vertex.tangent[3], -1.0);
    assert_eq!(vertex.color, [0.25, 0.5, 0.75, 1.0]);
}

#[test]
fn skin_mesh_asset_primitive_applies_morph_weights_before_skinning() {
    let mut mesh = MeshAsset::new(
        AssetUri::parse("res://meshes/morphed-skinned-direct.zmesh").unwrap(),
        RenderMeshTopology::PointList,
        BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[2.0, 0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[1.0, 0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_JOINT_INDEX.to_string(),
                MeshAttributeValues::Uint16x4(vec![[1, 0, 0, 0]]),
            ),
            (
                MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
                MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, 0.0]]),
            ),
        ]),
        Some(MeshIndices::U32(vec![0])),
    )
    .unwrap();
    mesh.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("Lift".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 1.0, 0.0]]),
        )]),
    }];
    let skeleton = unit_test_skeleton();
    let pose = joint_quarter_turn_pose();

    let skinned = skin_mesh_asset_primitive(&mesh, &skeleton, &pose, &[1.0])
        .expect("expected direct mesh morph weights to apply before CPU skinning");
    let vertex = &skinned.vertices[0];

    assert!(
        Vec3::from_array(vertex.position).abs_diff_eq(Vec3::new(0.0, 1.0, 0.0), 1.0e-4),
        "expected morphed position to be transformed by the posed joint"
    );
}

#[test]
fn prepare_skinned_mesh_asset_primitive_keeps_morphed_shader_source_before_cpu_skinning() {
    let mut mesh = MeshAsset::new(
        AssetUri::parse("res://meshes/morphed-skinned-source.zmesh").unwrap(),
        RenderMeshTopology::PointList,
        BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[2.0, 0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[1.0, 0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_JOINT_INDEX.to_string(),
                MeshAttributeValues::Uint16x4(vec![[1, 0, 0, 0]]),
            ),
            (
                MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
                MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, 0.0]]),
            ),
        ]),
        Some(MeshIndices::U32(vec![0])),
    )
    .unwrap();
    mesh.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("Lift".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 1.0, 0.0]]),
        )]),
    }];
    let skeleton = unit_test_skeleton();
    let pose = joint_quarter_turn_pose();

    let prepared = prepare_skinned_mesh_asset_primitive(&mesh, &skeleton, &pose, &[1.0])
        .expect("expected direct mesh morph weights to prepare CPU and shader-skinning paths");

    assert!(
        prepared.joint_palette_storage.is_some(),
        "shader skinning needs a GPU-visible joint palette"
    );
    assert!(
        Vec3::from_array(prepared.shader_skinning_source_primitive.vertices[0].position)
            .abs_diff_eq(Vec3::new(2.0, 1.0, 0.0), 1.0e-4),
        "shader source should keep the morphed but unskinned vertex"
    );
    assert!(
        Vec3::from_array(prepared.primitive.vertices[0].position)
            .abs_diff_eq(Vec3::new(0.0, 1.0, 0.0), 1.0e-4),
        "CPU fallback primitive should still apply skinning after morphing"
    );
}

fn unit_test_skeleton() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("unit-test-skeleton".to_string()),
        bones: vec![
            AnimationSkeletonBoneAsset {
                name: "root".to_string(),
                parent_index: None,
                local_translation: Vec3::ZERO.to_array(),
                local_rotation: Quat::IDENTITY.to_array(),
                local_scale: Vec3::ONE.to_array(),
            },
            AnimationSkeletonBoneAsset {
                name: "joint".to_string(),
                parent_index: Some(0),
                local_translation: Vec3::X.to_array(),
                local_rotation: Quat::IDENTITY.to_array(),
                local_scale: Vec3::ONE.to_array(),
            },
        ],
    }
}

fn oversized_storage_skeleton() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("oversized-uniform-skeleton".to_string()),
        bones: (0..=SKINNED_MESH_MAX_JOINT_MATRICES)
            .map(|index| AnimationSkeletonBoneAsset {
                name: format!("joint-{index}"),
                parent_index: None,
                local_translation: Vec3::ZERO.to_array(),
                local_rotation: Quat::IDENTITY.to_array(),
                local_scale: Vec3::ONE.to_array(),
            })
            .collect(),
    }
}

fn joint_quarter_turn_pose() -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones: vec![
            AnimationPoseBone {
                name: "root".to_string(),
                local_transform: Transform::identity(),
            },
            AnimationPoseBone {
                name: "joint".to_string(),
                local_transform: Transform::from_translation(Vec3::X)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            },
        ],
    }
}
