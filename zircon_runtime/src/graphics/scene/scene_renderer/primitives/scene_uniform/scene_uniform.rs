use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
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
}
