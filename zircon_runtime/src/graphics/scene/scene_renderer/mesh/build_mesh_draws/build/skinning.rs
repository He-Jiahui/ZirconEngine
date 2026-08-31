use std::collections::HashMap;

use crate::asset::{MeshAsset, ModelPrimitiveAsset};
use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::animation::AnimationSkeletonAsset;
use crate::core::math::{Mat4, Transform, Vec3};
use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteStorage;

#[derive(Clone, Debug)]
pub(super) struct SkinnedMeshJointPalette {
    joint_matrices: Vec<Mat4>,
}

impl SkinnedMeshJointPalette {
    pub(super) fn from_skeleton_pose(
        skeleton: &AnimationSkeletonAsset,
        pose: &AnimationPoseOutput,
    ) -> Result<Self, String> {
        let joint_matrices = build_joint_matrices(skeleton, pose)?;
        Ok(Self { joint_matrices })
    }

    pub(super) fn matrices(&self) -> &[Mat4] {
        &self.joint_matrices
    }

    pub(super) fn to_storage(&self) -> Result<SkinnedMeshJointPaletteStorage, String> {
        SkinnedMeshJointPaletteStorage::from_matrices(&self.joint_matrices)
    }
}

#[derive(Clone, Debug)]
pub(super) struct SkinnedMeshPreparedPrimitive {
    pub(super) primitive: ModelPrimitiveAsset,
    pub(super) shader_skinning_source_primitive: ModelPrimitiveAsset,
    pub(super) joint_palette_storage: Option<SkinnedMeshJointPaletteStorage>,
}

pub(super) fn skin_mesh_asset_primitive(
    mesh: &MeshAsset,
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
    morph_weights: &[f32],
) -> Result<ModelPrimitiveAsset, String> {
    let primitive = mesh
        .to_morphed_model_primitive(morph_weights)
        .map_err(|error| error.to_string())?;
    skin_model_primitive(&primitive, skeleton, pose)
}

pub(super) fn prepare_skinned_mesh_asset_primitive(
    mesh: &MeshAsset,
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
    morph_weights: &[f32],
) -> Result<SkinnedMeshPreparedPrimitive, String> {
    let primitive = mesh
        .to_morphed_model_primitive(morph_weights)
        .map_err(|error| error.to_string())?;
    prepare_skinned_model_primitive(&primitive, skeleton, pose)
}

pub(super) fn skin_model_primitive(
    primitive: &ModelPrimitiveAsset,
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<ModelPrimitiveAsset, String> {
    let joint_palette = SkinnedMeshJointPalette::from_skeleton_pose(skeleton, pose)?;

    Ok(skin_model_primitive_with_palette(primitive, &joint_palette))
}

pub(super) fn prepare_skinned_model_primitive(
    primitive: &ModelPrimitiveAsset,
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<SkinnedMeshPreparedPrimitive, String> {
    let joint_palette = SkinnedMeshJointPalette::from_skeleton_pose(skeleton, pose)?;
    let joint_palette_storage = joint_palette.to_storage().ok();
    let source_primitive = primitive.clone();
    let primitive = skin_model_primitive_with_palette(primitive, &joint_palette);

    Ok(SkinnedMeshPreparedPrimitive {
        primitive,
        shader_skinning_source_primitive: source_primitive,
        joint_palette_storage,
    })
}

fn skin_model_primitive_with_palette(
    primitive: &ModelPrimitiveAsset,
    joint_palette: &SkinnedMeshJointPalette,
) -> ModelPrimitiveAsset {
    ModelPrimitiveAsset {
        vertices: primitive
            .vertices
            .iter()
            .copied()
            .map(|vertex| skin_vertex(vertex, joint_palette.matrices()))
            .collect(),
        indices: primitive.indices.clone(),
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    }
}

fn bind_transform(
    bone: &crate::core::framework::animation::AnimationSkeletonBoneAsset,
) -> Transform {
    Transform {
        translation: Vec3::from_array(bone.local_translation),
        rotation: crate::core::math::Quat::from_array(bone.local_rotation).normalize(),
        scale: Vec3::from_array(bone.local_scale),
    }
}

fn build_joint_matrices(
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<Vec<Mat4>, String> {
    let pose_by_name = pose
        .bones
        .iter()
        .map(|bone| (bone.name.as_str(), bone.local_transform))
        .collect::<HashMap<_, _>>();
    let mut bind_worlds = Vec::with_capacity(skeleton.bones.len());
    let mut posed_worlds = Vec::with_capacity(skeleton.bones.len());
    for (index, bone) in skeleton.bones.iter().enumerate() {
        let bind_local = bind_transform(bone);
        let posed_local = pose_by_name
            .get(bone.name.as_str())
            .copied()
            .unwrap_or(bind_local);
        bind_worlds.push(compose_world_matrix(
            &bind_worlds,
            bone,
            index,
            bind_local.matrix(),
        )?);
        posed_worlds.push(compose_world_matrix(
            &posed_worlds,
            bone,
            index,
            posed_local.matrix(),
        )?);
    }
    for (posed_world, bind_world) in posed_worlds.iter_mut().zip(bind_worlds) {
        *posed_world = *posed_world * bind_world.inverse();
    }
    Ok(posed_worlds)
}

fn compose_world_matrix(
    worlds: &[Mat4],
    bone: &crate::core::framework::animation::AnimationSkeletonBoneAsset,
    index: usize,
    local_matrix: Mat4,
) -> Result<Mat4, String> {
    bone.parent_index
        .map(|parent| {
            worlds
                .get(parent as usize)
                .copied()
                .ok_or_else(|| {
                    format!(
                        "bone '{}' at index {index} references missing parent {parent}",
                        bone.name
                    )
                })
                .map(|parent_world| parent_world * local_matrix)
        })
        .transpose()
        .map(|world| world.unwrap_or(local_matrix))
}

fn skin_vertex(
    vertex: crate::asset::MeshVertex,
    joint_matrices: &[Mat4],
) -> crate::asset::MeshVertex {
    let joint_weights = vertex.joint_weights;
    let weight_sum = joint_weights
        .iter()
        .enumerate()
        .filter_map(|(slot, weight)| {
            (joint_matrices
                .get(vertex.joint_indices[slot] as usize)
                .is_some()
                && *weight > f32::EPSILON)
                .then_some(*weight)
        })
        .sum::<f32>();
    if weight_sum <= f32::EPSILON {
        return vertex;
    }

    let source_position = Vec3::from_array(vertex.position);
    let source_normal = Vec3::from_array(vertex.normal);
    let source_tangent =
        Vec3::from_array([vertex.tangent[0], vertex.tangent[1], vertex.tangent[2]]);
    let mut skinned_position = Vec3::ZERO;
    let mut skinned_normal = Vec3::ZERO;
    let mut skinned_tangent = Vec3::ZERO;
    for slot in 0..4 {
        let weight = joint_weights[slot];
        if weight <= f32::EPSILON {
            continue;
        }
        let Some(joint_matrix) = joint_matrices.get(vertex.joint_indices[slot] as usize) else {
            continue;
        };
        let normalized_weight = weight / weight_sum;
        skinned_position += joint_matrix.transform_point3(source_position) * normalized_weight;
        skinned_normal += joint_matrix.transform_vector3(source_normal) * normalized_weight;
        skinned_tangent += joint_matrix.transform_vector3(source_tangent) * normalized_weight;
    }

    let tangent = if skinned_tangent.length_squared() <= f32::EPSILON {
        source_tangent.normalize_or_zero()
    } else {
        skinned_tangent.normalize_or_zero()
    };
    let tangent_handedness = if vertex.tangent[3].abs() <= f32::EPSILON {
        1.0
    } else {
        vertex.tangent[3].signum()
    };
    let tangent_xyz = tangent.to_array();
    crate::asset::MeshVertex {
        position: skinned_position.to_array(),
        normal: skinned_normal.normalize_or_zero().to_array(),
        tangent: [
            tangent_xyz[0],
            tangent_xyz[1],
            tangent_xyz[2],
            tangent_handedness,
        ],
        ..vertex
    }
}

#[cfg(test)]
mod optimization_batch_de_runtime412_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::animation::{
        AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource, AnimationSkeletonAsset,
        AnimationSkeletonBoneAsset,
    };
    use crate::core::math::{Quat, Vec3};

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const BUILDS_PER_SAMPLE: usize = 96;
    const BONE_COUNT: usize = 256;

    #[test]
    fn optimization_batch_de_runtime412_fused_joint_palette_matches_legacy_staged_build() {
        let (skeleton, pose) = benchmark_skeleton_and_pose();

        assert_eq!(
            build_joint_matrices(&skeleton, &pose).unwrap(),
            legacy_joint_matrices(&skeleton, &pose).unwrap()
        );
    }

    #[test]
    fn optimization_batch_de_runtime412_fused_joint_palette_preserves_missing_parent_errors() {
        let (mut skeleton, pose) = benchmark_skeleton_and_pose();
        skeleton.bones[1].parent_index = Some(BONE_COUNT as u32);

        let error = build_joint_matrices(&skeleton, &pose).unwrap_err();

        assert!(error.contains("at index 1 references missing parent 256"));
    }

    #[test]
    fn optimization_batch_de_runtime412_fused_joint_palette_uses_two_bone_sized_world_buffers() {
        const SOURCE: &str = include_str!("skinning.rs");
        let production = SOURCE.split("#[cfg(test)]").next().unwrap();
        let build = production
            .split("fn build_joint_matrices")
            .nth(1)
            .unwrap()
            .split("fn compose_world_matrix")
            .next()
            .unwrap();

        assert_eq!(
            build
                .matches("Vec::with_capacity(skeleton.bones.len())")
                .count(),
            2
        );
        assert!(!build.contains("collect::<Vec<_>>()"));
        assert!(build.contains("posed_worlds.iter_mut().zip(bind_worlds)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_de_runtime412_skinning_palette_fused_build_p95() {
        let (skeleton, pose) = benchmark_skeleton_and_pose();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&skeleton, &pose, false));
                optimized.push(measure(&skeleton, &pose, true));
            } else {
                optimized.push(measure(&skeleton, &pose, true));
                legacy.push(measure(&skeleton, &pose, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME412_SKINNING_PALETTE_FUSED_BUILD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} builds_per_sample={BUILDS_PER_SAMPLE} bones_per_build={BONE_COUNT} legacy_bone_vectors_per_build=5 optimized_bone_vectors_per_build=2 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "fused palette construction must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn benchmark_skeleton_and_pose() -> (AnimationSkeletonAsset, AnimationPoseOutput) {
        let bones = (0..BONE_COUNT)
            .map(|index| AnimationSkeletonBoneAsset {
                name: format!("joint-{index}"),
                parent_index: (index > 0).then_some((index - 1) as u32),
                local_translation: [0.01, 0.02, 0.03],
                local_rotation: Quat::IDENTITY.to_array(),
                local_scale: Vec3::ONE.to_array(),
            })
            .collect::<Vec<_>>();
        let pose_bones = (0..BONE_COUNT)
            .step_by(2)
            .map(|index| AnimationPoseBone {
                name: format!("joint-{index}"),
                local_transform: Transform::from_translation(Vec3::new(0.02, 0.01, 0.03)),
            })
            .collect();
        (
            AnimationSkeletonAsset {
                name: Some("optimization-runtime412".to_string()),
                bones,
            },
            AnimationPoseOutput {
                source: AnimationPoseSource::Clip,
                active_state: None,
                bones: pose_bones,
            },
        )
    }

    fn legacy_joint_matrices(
        skeleton: &AnimationSkeletonAsset,
        pose: &AnimationPoseOutput,
    ) -> Result<Vec<Mat4>, String> {
        let bind_locals = skeleton
            .bones
            .iter()
            .map(bind_transform)
            .collect::<Vec<_>>();
        let pose_by_name = pose
            .bones
            .iter()
            .map(|bone| (bone.name.as_str(), bone.local_transform))
            .collect::<HashMap<_, _>>();
        let pose_locals = skeleton
            .bones
            .iter()
            .map(|bone| {
                pose_by_name
                    .get(bone.name.as_str())
                    .copied()
                    .unwrap_or_else(|| bind_transform(bone))
            })
            .collect::<Vec<_>>();
        let bind_worlds = legacy_world_matrices(skeleton, &bind_locals)?;
        let posed_worlds = legacy_world_matrices(skeleton, &pose_locals)?;
        Ok(bind_worlds
            .into_iter()
            .zip(posed_worlds)
            .map(|(bind_world, posed_world)| posed_world * bind_world.inverse())
            .collect())
    }

    fn legacy_world_matrices(
        skeleton: &AnimationSkeletonAsset,
        locals: &[Transform],
    ) -> Result<Vec<Mat4>, String> {
        let mut worlds = Vec::with_capacity(locals.len());
        for (index, (bone, local)) in skeleton.bones.iter().zip(locals).enumerate() {
            let world = compose_world_matrix(&worlds, bone, index, local.matrix())?;
            worlds.push(world);
        }
        Ok(worlds)
    }

    fn measure(
        skeleton: &AnimationSkeletonAsset,
        pose: &AnimationPoseOutput,
        optimized: bool,
    ) -> u128 {
        let started = Instant::now();
        for _ in 0..BUILDS_PER_SAMPLE {
            let matrices = if optimized {
                build_joint_matrices(black_box(skeleton), black_box(pose))
            } else {
                legacy_joint_matrices(black_box(skeleton), black_box(pose))
            }
            .unwrap();
            black_box(matrices);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests;
