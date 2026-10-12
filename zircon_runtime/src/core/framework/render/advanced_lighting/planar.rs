//! 平面反射探针只提供相机推导和按探针 ID 的捕获状态；实际渲染目标与提交顺序由相机循环管理。
#[path = "../planar/derive_camera.rs"]
mod derive_camera;
#[path = "../planar/oblique_projection.rs"]
mod oblique_projection;
#[path = "planar/probe_data.rs"]
mod probe_data;
#[path = "planar/quality.rs"]
mod quality;
#[path = "../planar/reflection_matrix.rs"]
mod reflection_matrix;
#[path = "planar/update_mode.rs"]
mod update_mode;
#[path = "../planar/update_state.rs"]
mod update_state;

pub use derive_camera::derive_planar_reflection_camera;
pub use oblique_projection::planar_oblique_near_clip_projection;
pub use probe_data::PlanarReflectionProbeData;
pub use quality::PlanarReflectionQuality;
pub use reflection_matrix::planar_reflection_matrix;
pub use update_mode::PlanarUpdateMode;
pub use update_state::PlanarReflectionUpdateState;

pub(super) const PLANAR_PLANE_EPSILON: f32 = 1.0e-6;

#[cfg(test)]
#[path = "planar/tests/cases.rs"]
mod tests;
