mod derive_camera;
mod oblique_projection;
mod probe_data;
mod quality;
mod reflection_matrix;
mod update_mode;
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
mod tests;
