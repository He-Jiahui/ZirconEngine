use std::collections::BTreeMap;

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
        let pose_by_name = pose
            .bones
            .iter()
            .map(|bone| (bone.name.as_str(), bone.local_transform))
            .collect::<BTreeMap<_, _>>();
        let bind_locals = skeleton
            .bones
            .iter()
            .map(bind_transform)
            .collect::<Vec<_>>();
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
