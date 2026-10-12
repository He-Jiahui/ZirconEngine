use crate::core::framework::render::MotionVectorCameraStatus;

/// 帧提交汇集的资源可用性快照，供效果报告解释运动向量、SSR 历史等降级原因。
/// 它不是效果开关；创作设置和实际资源状态须分别传入诊断构造函数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderPostProcessEffectStackResourceStatus {
    pub ssr_normal_available: bool,
    pub ssr_temporal_history_available: bool,
    pub motion_vector_available: bool,
    pub motion_vector_camera_available: bool,
    pub motion_vector_object_available: bool,
    pub motion_vector_tile_max_available: bool,
    pub motion_vector_tile_max_coarse_available: bool,
    pub motion_vector_neighbor_max_available: bool,
    pub motion_vector_camera_status: MotionVectorCameraStatus,
    pub motion_vector_prepass_available: bool,
}
