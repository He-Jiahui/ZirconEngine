use crate::core::framework::animation::AnimationPoseHandle;
use crate::core::resource::ResourceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RenderSceneSkeletalPoseIssue {
    NonFiniteTranslation,
    NonFiniteRotation,
    NonFiniteScale,
}

/// 持有已封存的动画姿态输入及其骨架身份，让 RenderScene 日志可跨后续动画更新保持旧帧语义。
/// 投影入口只校验数值有效性；骨骼矩阵求值交给后续网格绘制准备。
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSceneSkeletalPose {
    skeleton: ResourceId,
    pose: AnimationPoseHandle,
}

impl RenderSceneSkeletalPose {
    pub(crate) fn new(skeleton: ResourceId, pose: AnimationPoseHandle) -> Self {
        Self { skeleton, pose }
    }

    pub(crate) const fn skeleton(&self) -> &ResourceId {
        &self.skeleton
    }

    pub(crate) const fn pose(&self) -> &AnimationPoseHandle {
        &self.pose
    }

    pub(super) fn validate(&self) -> Result<(), RenderSceneSkeletalPoseIssue> {
        for bone in &self.pose.bones {
            let transform = &bone.local_transform;
            if transform
                .translation
                .to_array()
                .into_iter()
                .any(|value| !value.is_finite())
            {
                return Err(RenderSceneSkeletalPoseIssue::NonFiniteTranslation);
            }
            if transform
                .rotation
                .to_array()
                .into_iter()
                .any(|value| !value.is_finite())
            {
                return Err(RenderSceneSkeletalPoseIssue::NonFiniteRotation);
            }
            if transform
                .scale
                .to_array()
                .into_iter()
                .any(|value| !value.is_finite())
            {
                return Err(RenderSceneSkeletalPoseIssue::NonFiniteScale);
            }
        }
        Ok(())
    }
}
