use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct SceneUniform {
    /// Current frame clip-from-world matrix. This may include temporal jitter.
    pub(crate) view_proj: [[f32; 4]; 4],
    /// Current frame clip-from-world matrix with temporal jitter removed.
    pub(crate) view_proj_unjittered: [[f32; 4]; 4],
    /// Current frame world-from-clip matrix with temporal jitter removed.
    pub(crate) inverse_view_proj: [[f32; 4]; 4],
    pub(crate) ambient_color: [f32; 4],
    /// Previous frame clip-from-world matrix with temporal jitter removed.
    pub(crate) previous_view_proj_unjittered: [[f32; 4]; 4],
    pub(crate) motion_params: [f32; 4],
    /// xy = pixel jitter offset, z = Halton sequence index, w = active jitter flag.
    pub(crate) jitter_params: [f32; 4],
    pub(crate) sky_horizon_color: [f32; 4],
    pub(crate) sky_zenith_color: [f32; 4],
    pub(crate) sky_ground_color: [f32; 4],
    /// x = skybox enabled, y = sky intensity, z = sky rotation radians, w = IBL enabled.
    pub(crate) environment_params: [f32; 4],
    /// x = environment source kind, y = base sample width, z = base sample height, w = mip count.
    pub(crate) environment_sample_params: [f32; 4],
}

impl Default for SceneUniform {
    fn default() -> Self {
        Self {
            view_proj: [[0.0; 4]; 4],
            view_proj_unjittered: [[0.0; 4]; 4],
            inverse_view_proj: [[0.0; 4]; 4],
            ambient_color: [0.0; 4],
            previous_view_proj_unjittered: [[0.0; 4]; 4],
            motion_params: [0.0; 4],
            jitter_params: [0.0; 4],
            sky_horizon_color: [0.0; 4],
            sky_zenith_color: [0.0; 4],
            sky_ground_color: [0.0; 4],
            environment_params: [0.0; 4],
            environment_sample_params: [0.0; 4],
        }
    }
}
