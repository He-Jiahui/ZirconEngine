use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::{MeshAsset, ModelPrimitiveAsset};
use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::animation::AnimationSkeletonAsset;
use crate::core::math::{Mat4, Transform, Vec3};
use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteStorage;

/// 骨架绑定姿态与动画局部姿态合成的每关节形变矩阵。
/// 绘制项保留 CPU 结果和 GPU 原始源；固定 GPU 调色板超限时仍可绘制 CPU 结果。
#[derive(Clone, Debug)]
pub(super) struct SkinnedMeshJointPalette {
    joint_matrices: Vec<Mat4>,
    storage: Option<Arc<SkinnedMeshJointPaletteStorage>>,
}

impl SkinnedMeshJointPalette {
    pub(super) fn from_skeleton_pose(
        skeleton: &AnimationSkeletonAsset,
        pose: &AnimationPoseOutput,
    ) -> Result<Self, String> {
        let joint_matrices = build_joint_matrices(skeleton, pose)?;
        let storage = SkinnedMeshJointPaletteStorage::from_matrices(&joint_matrices)
            .ok()
            .map(Arc::new);
        Ok(Self {
            joint_matrices,
            storage,
        })
    }

    pub(super) fn matrices(&self) -> &[Mat4] {
        &self.joint_matrices
    }

    pub(super) fn to_storage(&self) -> Option<Arc<SkinnedMeshJointPaletteStorage>> {
        self.storage.clone()
    }
}

#[derive(Clone, Debug)]
pub(super) struct SkinnedMeshPreparedPrimitive {
    pub(super) primitive: ModelPrimitiveAsset,
    pub(super) shader_skinning_source_primitive: ModelPrimitiveAsset,
    pub(super) joint_palette_storage: Option<Arc<SkinnedMeshJointPaletteStorage>>,
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
    let joint_palette = SkinnedMeshJointPalette::from_skeleton_pose(skeleton, pose)?;
    Ok(skin_owned_model_primitive_with_palette(
        primitive,
        &joint_palette,
    ))
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
    let joint_palette = SkinnedMeshJointPalette::from_skeleton_pose(skeleton, pose)?;
    Ok(prepare_skinned_owned_model_primitive(
        primitive,
        &joint_palette,
    ))
}

pub(super) fn skin_model_primitive(
    primitive: &ModelPrimitiveAsset,
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<ModelPrimitiveAsset, String> {
    let joint_palette = SkinnedMeshJointPalette::from_skeleton_pose(skeleton, pose)?;

    Ok(skin_model_primitive_with_palette(primitive, &joint_palette))
}

/// 构造 CPU 回退网格并保留未蒙皮源，供可用的 GPU 蒙皮路径读取。
/// 两份数据必须对应同一骨架/姿态；GPU 存储失败仅取消 GPU 路径。
#[cfg(test)]
pub(super) fn prepare_skinned_model_primitive(
    primitive: &ModelPrimitiveAsset,
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<SkinnedMeshPreparedPrimitive, String> {
    let joint_palette = SkinnedMeshJointPalette::from_skeleton_pose(skeleton, pose)?;
    Ok(prepare_skinned_owned_model_primitive(
        primitive.clone(),
        &joint_palette,
    ))
}

#[cfg(test)]
pub(super) fn prepare_skinned_model_primitive_with_palette(
    primitive: &ModelPrimitiveAsset,
    joint_palette: &SkinnedMeshJointPalette,
) -> SkinnedMeshPreparedPrimitive {
    prepare_skinned_owned_model_primitive(primitive.clone(), joint_palette)
}

fn prepare_skinned_owned_model_primitive(
    source_primitive: ModelPrimitiveAsset,
    joint_palette: &SkinnedMeshJointPalette,
) -> SkinnedMeshPreparedPrimitive {
    let joint_palette_storage = joint_palette.to_storage();
    let primitive = skin_model_primitive_with_palette(&source_primitive, joint_palette);

    SkinnedMeshPreparedPrimitive {
        primitive,
        shader_skinning_source_primitive: source_primitive,
        joint_palette_storage,
    }
}

pub(super) fn skin_model_primitive_with_palette(
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

// The direct-mesh conversion already owns its buffers; CPU skinning can reuse them.
fn skin_owned_model_primitive_with_palette(
    primitive: ModelPrimitiveAsset,
    joint_palette: &SkinnedMeshJointPalette,
) -> ModelPrimitiveAsset {
    let ModelPrimitiveAsset {
        mut vertices,
        indices,
        ..
    } = primitive;
    for vertex in &mut vertices {
        *vertex = skin_vertex(*vertex, joint_palette.matrices());
    }
    ModelPrimitiveAsset {
        vertices,
        indices,
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
    let mut blended_x = Vec3::ZERO;
    let mut blended_y = Vec3::ZERO;
    let mut blended_z = Vec3::ZERO;
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
        blended_x += joint_matrix.x_axis.truncate() * normalized_weight;
        blended_y += joint_matrix.y_axis.truncate() * normalized_weight;
        blended_z += joint_matrix.z_axis.truncate() * normalized_weight;
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
        normal: blended_skin_normal(blended_x, blended_y, blended_z, source_normal).to_array(),
        tangent: [
            tangent_xyz[0],
            tangent_xyz[1],
            tangent_xyz[2],
            tangent_handedness,
        ],
        ..vertex
    }
}

// Cofactors give the direction of inverse-transpose(A) without dividing by a
// potentially tiny determinant. Scale both operands first to keep all products finite.
// A singular or unrepresentable direction has no reliable normal and becomes zero.
fn blended_skin_normal(x: Vec3, y: Vec3, z: Vec3, normal: Vec3) -> Vec3 {
    if !x.is_finite() || !y.is_finite() || !z.is_finite() || !normal.is_finite() {
        return Vec3::ZERO;
    }
    let scale = x.abs().max(y.abs()).max(z.abs()).max_element();
    let normal_scale = normal.abs().max_element();
    if scale <= 0.0 || normal_scale <= 0.0 {
        return Vec3::ZERO;
    }
    let x = x / scale;
    let y = y / scale;
    let z = z / scale;
    let determinant = x.dot(y.cross(z));
    if determinant == 0.0 || !determinant.is_finite() {
        return Vec3::ZERO;
    }
    let normal = normal / normal_scale;
    let cofactor_normal = (y.cross(z) * normal.x + z.cross(x) * normal.y + x.cross(y) * normal.z)
        * determinant.signum();
    let direction_scale = cofactor_normal.abs().max_element();
    if direction_scale <= 0.0 || !direction_scale.is_finite() {
        return Vec3::ZERO;
    }
    (cofactor_normal / direction_scale).normalize_or_zero()
}

#[cfg(test)]
#[path = "tests/skinning_perf.rs"]
mod optimization_batch_de_runtime412_tests;

#[cfg(test)]
#[path = "skinning/tests/cases.rs"]
mod tests;

#[cfg(all(test, feature = "memory-validation"))]
#[path = "skinning/tests/model_memory.rs"]
mod model_memory;
