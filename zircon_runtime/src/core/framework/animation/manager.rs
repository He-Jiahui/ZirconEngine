use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;

use super::{
    AnimationClipAsset, AnimationGpuSkinningReadiness, AnimationGraphAsset,
    AnimationGraphEvaluation, AnimationParameterMap, AnimationParameterValue,
    AnimationPlaybackSettings, AnimationPoseOutput, AnimationResult, AnimationRuntimeStatus,
    AnimationSkeletonAsset, AnimationStateMachineAsset, AnimationStateMachineEvaluation,
    AnimationTickReport, AnimationTickRequest, AnimationTrackPath,
};

/// Core 服务注册表暴露的动画能力边界；实现负责配置、参数和单次求值，
/// 场景中的逐帧调度、资源缓存与 World 写入由调用方的运行时流水线协调。
pub trait AnimationManager: Send + Sync {
    fn playback_settings(&self) -> AnimationPlaybackSettings;
    fn normalize_track_path(&self, path: &AnimationTrackPath) -> AnimationTrackPath;
    fn parameter_defaults(&self, graph: &AnimationGraphAsset) -> AnimationParameterMap;
    fn parameter_value(
        &self,
        parameters: &AnimationParameterMap,
        name: &str,
    ) -> Option<AnimationParameterValue>;
    fn set_parameter(
        &self,
        parameters: &mut AnimationParameterMap,
        name: &str,
        value: AnimationParameterValue,
    );
    fn evaluate_graph(
        &self,
        graph: &AnimationGraphAsset,
        parameters: &AnimationParameterMap,
    ) -> AnimationGraphEvaluation;
    fn evaluate_state_machine(
        &self,
        state_machine: &AnimationStateMachineAsset,
        current_state: Option<&str>,
        parameters: &AnimationParameterMap,
    ) -> AnimationStateMachineEvaluation;
    /// 在调用方已取得骨架与片段后采样局部骨骼姿态；资源加载与姿态发布不属于此接口。
    fn sample_clip_pose(
        &self,
        skeleton: &AnimationSkeletonAsset,
        clip: &AnimationClipAsset,
        time_seconds: Real,
        looping: bool,
    ) -> AnimationResult<AnimationPoseOutput>;
    // TODO: [CR-ANIMATION-0001] 核实世界 tick/status 的服务职责：当前两套管理器实现沿用空默认值且无生产调用；接入前需确定真实帧状态来源。
    fn tick_world_contract(&self, request: AnimationTickRequest) -> AnimationTickReport {
        AnimationTickReport::new(request.world)
    }
    fn runtime_status(&self, world: WorldHandle) -> AnimationRuntimeStatus {
        AnimationRuntimeStatus::new(world)
    }
    fn gpu_skinning_readiness(&self) -> AnimationGpuSkinningReadiness {
        AnimationGpuSkinningReadiness::default()
    }
}
