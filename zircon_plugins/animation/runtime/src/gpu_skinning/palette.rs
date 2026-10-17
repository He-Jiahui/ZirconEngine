//! 由骨架绑定姿态与动画局部姿态生成关节矩阵；输出行序必须与骨架和网格关节索引一致。
//! 按名称回退只适用于唯一骨名，完整姿态优先按骨架顺序读取。
use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::framework::animation::{
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::math::{Mat4, Quat, Transform, Vec3};

use super::SkinningPaletteError;

pub const MAX_SKIN_JOINTS: usize = 256;

#[derive(Clone, Debug, PartialEq)]
pub struct SkinningPalette {
    pub joint_matrices: Box<[Mat4]>,
}

impl Default for SkinningPalette {
    fn default() -> Self {
        Self {
            joint_matrices: Box::new([]),
        }
    }
}

impl SkinningPalette {
    /// 输入骨架须以父骨先于子骨的顺序排列，绑定矩阵还需可逆；姿态缺行使用绑定变换。
    pub fn from_skeleton_pose(
        skeleton: &AnimationSkeletonAsset,
        pose: &AnimationPoseOutput,
    ) -> Result<Self, SkinningPaletteError> {
        if skeleton.bones.len() > MAX_SKIN_JOINTS {
            return Err(SkinningPaletteError::TooManyJoints {
                joint_count: skeleton.bones.len(),
                limit: MAX_SKIN_JOINTS,
            });
        }
        let bind_locals = skeleton
            .bones
            .iter()
            .map(bind_transform)
            .collect::<Vec<_>>();
        let pose_locals = pose_locals_for_skeleton(skeleton, pose)?;
        let bind_world = compose_world(skeleton, &bind_locals)?;
        let pose_world = compose_world(skeleton, &pose_locals)?;
        let joint_matrices = bind_world
            .into_iter()
            .zip(pose_world)
            .map(|(bind, posed)| posed * bind.inverse())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(Self { joint_matrices })
    }

    pub fn joint_count(&self) -> usize {
        self.joint_matrices.len()
    }
}

fn pose_locals_for_skeleton(
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<Vec<Transform>, SkinningPaletteError> {
    if pose.bones.len() == skeleton.bones.len()
        && pose
            .bones
            .iter()
            .zip(&skeleton.bones)
            .all(|(pose_bone, skeleton_bone)| pose_bone.name == skeleton_bone.name)
    {
        return Ok(pose.bones.iter().map(|bone| bone.local_transform).collect());
    }

    // 完整同序姿态按索引对应；名称回退必须能唯一确定姿态和骨架中的同一骨骼。
    let mut pose_by_name = BTreeMap::new();
    for bone in &pose.bones {
        if pose_by_name
            .insert(bone.name.as_str(), bone.local_transform)
            .is_some()
        {
            return Err(SkinningPaletteError::AmbiguousPoseBone {
                bone: bone.name.clone(),
            });
        }
    }
    let mut matched_names = BTreeSet::new();
    skeleton
        .bones
        .iter()
        .map(|bone| {
            if let Some(transform) = pose_by_name.get(bone.name.as_str()).copied() {
                if !matched_names.insert(bone.name.as_str()) {
                    return Err(SkinningPaletteError::AmbiguousPoseBone {
                        bone: bone.name.clone(),
                    });
                }
                Ok(transform)
            } else {
                Ok(bind_transform(bone))
            }
        })
        .collect()
}

fn bind_transform(bone: &AnimationSkeletonBoneAsset) -> Transform {
    Transform {
        translation: Vec3::from_array(bone.local_translation),
        rotation: Quat::from_array(bone.local_rotation).normalize(),
        scale: Vec3::from_array(bone.local_scale),
    }
}

fn compose_world(
    skeleton: &AnimationSkeletonAsset,
    locals: &[Transform],
) -> Result<Vec<Mat4>, SkinningPaletteError> {
    let mut worlds = Vec::<Mat4>::with_capacity(locals.len());
    for (bone, local) in skeleton.bones.iter().zip(locals) {
        let world = match bone.parent_index {
            Some(parent) => {
                worlds.get(parent as usize).copied().ok_or_else(|| {
                    SkinningPaletteError::MissingParent {
                        bone: bone.name.clone(),
                        parent_index: parent,
                    }
                })? * local.matrix()
            }
            None => local.matrix(),
        };
        worlds.push(world);
    }
    Ok(worlds)
}

#[cfg(test)]
#[path = "tests/palette_perf.rs"]
mod optimization_tests;
