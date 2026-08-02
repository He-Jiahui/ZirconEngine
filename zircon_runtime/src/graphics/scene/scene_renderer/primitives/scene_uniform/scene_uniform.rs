use bytemuck::{Pod, Zeroable};

const MAX_GLOBAL_MATERIAL_MIP_BIAS: f32 = 4.0;

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
    /// xyz = camera world position, w = global material texture mip bias.
    pub(crate) camera_world_position: [f32; 4],
    /// xyz = orthographic camera-to-view direction, w = orthographic flag.
    pub(crate) camera_view_direction: [f32; 4],
    pub(crate) sky_horizon_color: [f32; 4],
    pub(crate) sky_zenith_color: [f32; 4],
    pub(crate) sky_ground_color: [f32; 4],
    /// xyz = procedural sun direction, w = enabled flag.
    pub(crate) sky_sun_direction: [f32; 4],
    /// rgb = procedural sun radiance color, w = authored angular radius for diagnostics.
    pub(crate) sky_sun_color_radius: [f32; 4],
    /// x = sun intensity, y = outer cosine, z = inner cosine, w reserved.
    pub(crate) sky_sun_params: [f32; 4],
    /// x = source IEM available, y = sky intensity, z = sky rotation radians, w = IBL enabled.
    pub(crate) environment_params: [f32; 4],
    /// x = environment source kind, y = base sample width, z = base sample height, w = mip count.
    pub(crate) environment_sample_params: [f32; 4],
    /// x = environment rotation sine, y = cosine, z = nonzero rotation flag, w reserved.
    pub(crate) environment_rotation_sin_cos: [f32; 4],
}

impl SceneUniform {
    pub(in crate::graphics::scene::scene_renderer) fn set_global_material_mip_bias(
        &mut self,
        mip_bias: f32,
    ) {
        self.camera_world_position[3] = if mip_bias.is_finite() {
            mip_bias.clamp(0.0, MAX_GLOBAL_MATERIAL_MIP_BIAS)
        } else {
            0.0
        };
    }

    pub(in crate::graphics::scene::scene_renderer) fn use_realtime_ibl(
        &mut self,
        source_face_size: u32,
        pmrem_face_size: u32,
        pmrem_mip_count: u32,
    ) {
        self.environment_sample_params = [
            4.0,
            source_face_size.max(1) as f32,
            pmrem_face_size.max(1) as f32,
            pmrem_mip_count.max(1) as f32,
        ];
    }
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
            camera_world_position: [0.0; 4],
            camera_view_direction: [0.0; 4],
            sky_horizon_color: [0.0; 4],
            sky_zenith_color: [0.0; 4],
            sky_ground_color: [0.0; 4],
            sky_sun_direction: [0.0; 4],
            sky_sun_color_radius: [0.0; 4],
            sky_sun_params: [0.0; 4],
            environment_params: [0.0; 4],
            environment_sample_params: [0.0; 4],
            environment_rotation_sin_cos: [0.0, 1.0, 0.0, 0.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SceneUniform;

    #[test]
    fn render_perf_global_material_mip_bias_is_sanitized_before_gpu_upload() {
        let mut uniform = SceneUniform::default();

        uniform.set_global_material_mip_bias(1.0);
        assert_eq!(uniform.camera_world_position[3], 1.0);

        uniform.set_global_material_mip_bias(f32::NAN);
        assert_eq!(uniform.camera_world_position[3], 0.0);

        uniform.set_global_material_mip_bias(f32::MAX);
        assert_eq!(uniform.camera_world_position[3], 4.0);
    }
}
