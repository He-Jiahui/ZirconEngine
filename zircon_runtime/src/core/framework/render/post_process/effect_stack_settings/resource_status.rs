use crate::core::framework::render::MotionVectorCameraStatus;

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
