use std::collections::HashMap;

use crate::asset::{AnimationSkeletonAsset, ModelPrimitiveAsset};
use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::math::{Mat4, Transform, Vec3};

pub(super) fn skin_model_primitive(
    primitive: &ModelPrimitiveAsset,
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<ModelPrimitiveAsset, String> {
    let bind_local = skeleton
        .bones
        .iter()
        .map(bind_transform)
        .collect::<Vec<_>>();
    let pose_locals = pose_local_transforms(skeleton, pose);
    let bind_world = compose_world_matrices(skeleton, &bind_local)?;
    let posed_world = compose_world_matrices(skeleton, &pose_locals)?;
    let joint_matrices = bind_world
        .into_iter()
        .zip(posed_world)
        .map(|(bind_world, posed_world)| posed_world * bind_world.inverse())
        .collect::<Vec<_>>();

    Ok(ModelPrimitiveAsset {
        vertices: primitive
            .vertices
            .iter()
            .copied()
            .map(|vertex| skin_vertex(vertex, &joint_matrices))
            .collect(),
        indices: primitive.indices.clone(),
        virtual_geometry: None,
    })
}

fn bind_transform(bone: &crate::asset::AnimationSkeletonBoneAsset) -> Transform {
    Transform {
        translation: Vec3::from_array(bone.local_translation),
        rotation: crate::core::math::Quat::from_array(bone.local_rotation).normalize(),
        scale: Vec3::from_array(bone.local_scale),
    }
}

fn pose_local_transforms(
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Vec<Transform> {
    let pose_by_name = pose
        .bones
        .iter()
        .map(|bone| (bone.name.as_str(), bone.local_transform))
        .collect::<HashMap<_, _>>();
    skeleton
        .bones
        .iter()
        .map(|bone| {
            pose_by_name
                .get(bone.name.as_str())
                .copied()
                .unwrap_or_else(|| bind_transform(bone))
        })
        .collect()
}

fn compose_world_matrices(
    skeleton: &AnimationSkeletonAsset,
    locals: &[Transform],
) -> Result<Vec<Mat4>, String> {
    if locals.len() != skeleton.bones.len() {
        return Err(format!(
            "pose transform count {} does not match skeleton bone count {}",
            locals.len(),
            skeleton.bones.len()
        ));
    }

    let mut worlds = Vec::with_capacity(locals.len());
    for (index, (bone, local)) in skeleton.bones.iter().zip(locals.iter()).enumerate() {
        let local_matrix = local.matrix();
        let world = bone
            .parent_index
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
            .transpose()?
            .unwrap_or(local_matrix);
        worlds.push(world);
    }
    Ok(worlds)
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
    let mut skinned_position = Vec3::ZERO;
    let mut skinned_normal = Vec3::ZERO;
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
    }

    crate::asset::MeshVertex {
        position: skinned_position.to_array(),
        normal: skinned_normal.normalize_or_zero().to_array(),
        ..vertex
    }
}

#[cfg(test)]
mod tests {
    use super::skin_model_primitive;
    use crate::asset::{
        AnimationSkeletonAsset, AnimationSkeletonBoneAsset, MeshVertex, ModelPrimitiveAsset,
    };
    use crate::core::framework::animation::{
        AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
    };
    use crate::core::math::{Quat, Transform, Vec2, Vec3};

    #[test]
    fn skin_model_primitive_rotates_weighted_vertex_around_joint_bind_origin() {
        let primitive = ModelPrimitiveAsset {
            vertices: vec![
                MeshVertex::new(Vec3::new(2.0, 0.0, 0.0), Vec3::X, Vec2::ZERO)
                    .with_skinning([1, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]),
            ],
            indices: vec![0],
            virtual_geometry: None,
        };
        let skeleton = AnimationSkeletonAsset {
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
        };
        let pose = AnimationPoseOutput {
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
        };

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
    }
}
