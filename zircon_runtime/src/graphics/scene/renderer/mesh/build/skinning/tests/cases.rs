use std::collections::BTreeMap;

use super::{
    prepare_skinned_mesh_asset_primitive, prepare_skinned_model_primitive,
    prepare_skinned_owned_model_primitive, skin_mesh_asset_primitive, skin_model_primitive,
    skin_owned_model_primitive_with_palette, SkinnedMeshJointPalette,
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
use crate::graphics::scene::scene_renderer::mesh::skinning::{
    SkinnedMeshJointPaletteStorage, SKINNED_MESH_MAX_JOINT_MATRICES,
};

#[test]
fn owned_skinning_reuses_vertex_and_index_allocations() {
    let primitive = memory_test_primitive();
    let vertex_pointer = primitive.vertices.as_ptr();
    let index_pointer = primitive.indices.as_ptr();
    let palette = SkinnedMeshJointPalette::from_skeleton_pose(
        &unit_test_skeleton(),
        &joint_quarter_turn_pose(),
    )
    .unwrap();

    let skinned = skin_owned_model_primitive_with_palette(primitive, &palette);

    assert_eq!(skinned.vertices.as_ptr(), vertex_pointer);
    assert_eq!(skinned.indices.as_ptr(), index_pointer);
    assert_eq!(skinned.indices, [0]);
    assert!(Vec3::from_array(skinned.vertices[0].position)
        .abs_diff_eq(Vec3::new(1.0, 1.0, 0.0), 1.0e-4));
}

#[test]
fn owned_skinning_preparation_retains_source_allocations() {
    let primitive = memory_test_primitive();
    let vertex_pointer = primitive.vertices.as_ptr();
    let index_pointer = primitive.indices.as_ptr();
    let palette = SkinnedMeshJointPalette::from_skeleton_pose(
        &unit_test_skeleton(),
        &joint_quarter_turn_pose(),
    )
    .unwrap();

    let prepared = prepare_skinned_owned_model_primitive(primitive, &palette);

    assert_eq!(
        prepared.shader_skinning_source_primitive.vertices.as_ptr(),
        vertex_pointer
    );
    assert_eq!(
        prepared.shader_skinning_source_primitive.indices.as_ptr(),
        index_pointer
    );
    assert_eq!(
        prepared.shader_skinning_source_primitive.vertices[0].position,
        [2.0, 0.0, 0.0]
    );
    assert!(Vec3::from_array(prepared.primitive.vertices[0].position)
        .abs_diff_eq(Vec3::new(1.0, 1.0, 0.0), 1.0e-4));
    assert!(prepared.joint_palette_storage.is_some());
}

fn memory_test_primitive() -> ModelPrimitiveAsset {
    ModelPrimitiveAsset {
        vertices: vec![
            MeshVertex::new(Vec3::new(2.0, 0.0, 0.0), Vec3::X, Vec2::ZERO)
                .with_skinning([1, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]),
        ],
        indices: vec![0],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    }
}

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
        storage.joint_matrices(),
        &[
            palette.matrices()[0].to_cols_array_2d(),
            palette.matrices()[1].to_cols_array_2d(),
        ],
        "the snapshot contains exactly the active matrices, with no padded slots"
    );
}

#[test]
fn joint_palette_storage_rejects_current_storage_limit_overflow() {
    let joint_matrices =
        vec![crate::core::math::Mat4::IDENTITY; SKINNED_MESH_MAX_JOINT_MATRICES + 1];
    let error = SkinnedMeshJointPaletteStorage::from_matrices(&joint_matrices)
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

#[test]
fn multiple_primitives_share_the_same_pose_snapshot() {
    let palette = SkinnedMeshJointPalette::from_skeleton_pose(
        &unit_test_skeleton(),
        &joint_quarter_turn_pose(),
    )
    .unwrap();
    let first =
        super::prepare_skinned_model_primitive_with_palette(&memory_test_primitive(), &palette);
    let second =
        super::prepare_skinned_model_primitive_with_palette(&memory_test_primitive(), &palette);
    assert!(std::sync::Arc::ptr_eq(
        first.joint_palette_storage.as_ref().unwrap(),
        second.joint_palette_storage.as_ref().unwrap()
    ));
}

#[test]
fn cpu_skinning_normals_follow_the_inverse_transpose_of_the_blended_linear_transform() {
    let source_normal = Vec3::new(1.0, -1.0, 0.0).normalize();
    let source_tangent = Vec3::new(1.0, 1.0, 0.0).normalize();
    let source = MeshVertex::new(Vec3::new(1.0, 2.0, 0.0), source_normal, Vec2::ZERO)
        .with_tangent([source_tangent.x, source_tangent.y, source_tangent.z, -1.0]);
    let skeleton = AnimationSkeletonAsset {
        name: Some("blended-normal-contract".into()),
        bones: ["first", "second"]
            .into_iter()
            .map(|name| AnimationSkeletonBoneAsset {
                name: name.into(),
                parent_index: None,
                local_translation: Vec3::ZERO.to_array(),
                local_rotation: Quat::IDENTITY.to_array(),
                local_scale: Vec3::ONE.to_array(),
            })
            .collect(),
    };
    for (scales, weights, expected_normal, expected_position, expected_tangent) in [
        (
            [Vec3::new(2.0, 1.0, 1.0), Vec3::ONE],
            [1.0, 0.0, 0.0, 0.0],
            Vec3::new(0.5, -1.0, 0.0).normalize(),
            Vec3::new(2.0, 2.0, 0.0),
            Vec3::new(2.0, 1.0, 0.0).normalize(),
        ),
        (
            [Vec3::new(2.0, 1.0, 1.0), Vec3::new(1.0, 3.0, 1.0)],
            [0.5, 0.5, 0.0, 0.0],
            Vec3::new(2.0 / 3.0, -0.5, 0.0).normalize(),
            Vec3::new(1.5, 4.0, 0.0),
            Vec3::new(1.5, 2.0, 0.0).normalize(),
        ),
        (
            [Vec3::new(-2.0, 1.0, 1.0), Vec3::ONE],
            [1.0, 0.0, 0.0, 0.0],
            Vec3::new(-0.5, -1.0, 0.0).normalize(),
            Vec3::new(-2.0, 2.0, 0.0),
            Vec3::new(-2.0, 1.0, 0.0).normalize(),
        ),
        (
            [Vec3::new(0.0, 1.0, 1.0), Vec3::ONE],
            [1.0, 0.0, 0.0, 0.0],
            Vec3::ZERO,
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::Y,
        ),
    ] {
        let pose = AnimationPoseOutput {
            source: AnimationPoseSource::Clip,
            active_state: None,
            bones: ["first", "second"]
                .into_iter()
                .zip(scales)
                .map(|(name, scale)| AnimationPoseBone {
                    name: name.into(),
                    local_transform: Transform::identity().with_scale(scale),
                })
                .collect(),
        };
        let primitive = ModelPrimitiveAsset {
            vertices: vec![source.with_skinning([0, 1, 0, 0], weights)],
            indices: vec![0],
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: None,
        };
        let palette = SkinnedMeshJointPalette::from_skeleton_pose(&skeleton, &pose).unwrap();
        let prepared = prepare_skinned_model_primitive(&primitive, &skeleton, &pose).unwrap();
        let skinned = skin_model_primitive(&primitive, &skeleton, &pose).unwrap();
        let owned = skin_owned_model_primitive_with_palette(primitive.clone(), &palette);
        for vertex in [
            skinned.vertices[0],
            owned.vertices[0],
            prepared.primitive.vertices[0],
        ] {
            let normal = Vec3::from_array(vertex.normal);
            let tangent = Vec3::from_array(vertex.tangent[..3].try_into().unwrap());
            assert!(
                normal.is_finite() && normal.abs_diff_eq(expected_normal, 1.0e-5),
                "{normal:?}"
            );
            assert!(normal.dot(tangent).abs() <= 1.0e-5);
            assert!(Vec3::from_array(vertex.position).abs_diff_eq(expected_position, 1.0e-5));
            assert!(tangent.abs_diff_eq(expected_tangent, 1.0e-5));
            assert_eq!(vertex.tangent[3], -1.0);
        }
        assert_eq!(
            prepared.shader_skinning_source_primitive.vertices,
            primitive.vertices
        );
    }
}

#[test]
fn cpu_skinning_normal_fallback_is_finite_without_changing_invalid_joint_or_zero_weight_paths() {
    let source = MeshVertex::new(Vec3::X, Vec3::Y, Vec2::ZERO)
        .with_skinning([99, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]);
    let primitive = ModelPrimitiveAsset {
        vertices: vec![source],
        indices: vec![0],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };
    let palette = SkinnedMeshJointPalette::from_skeleton_pose(
        &unit_test_skeleton(),
        &joint_quarter_turn_pose(),
    )
    .unwrap();
    assert_eq!(
        skin_owned_model_primitive_with_palette(primitive.clone(), &palette).vertices,
        primitive.vertices
    );
    let mut zero_weight = primitive.clone();
    zero_weight.vertices[0].joint_indices = [0; 4];
    zero_weight.vertices[0].joint_weights = [0.0; 4];
    assert_eq!(
        skin_owned_model_primitive_with_palette(zero_weight.clone(), &palette).vertices,
        zero_weight.vertices
    );
    let mut invalid_normal = primitive;
    invalid_normal.vertices[0].joint_indices = [0; 4];
    invalid_normal.vertices[0].normal = [f32::INFINITY, 1.0, 0.0];
    let result = skin_owned_model_primitive_with_palette(invalid_normal, &palette);
    assert_eq!(result.vertices[0].normal, [0.0; 3]);
    assert_eq!(result.vertices[0].position, Vec3::X.to_array());
}
