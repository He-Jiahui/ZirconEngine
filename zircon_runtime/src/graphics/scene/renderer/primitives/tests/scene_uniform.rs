use super::SceneUniform;
use std::mem::{offset_of, size_of};

#[test]
fn scene_uniform_ambient_policy_preserves_the_wgsl_byte_layout() {
    assert_eq!(offset_of!(SceneUniform, ambient_color), 192);
    assert_eq!(offset_of!(SceneUniform, lightmapped_ambient_color), 208);
    assert_eq!(offset_of!(SceneUniform, previous_view_proj_unjittered), 224);
    assert_eq!(size_of::<SceneUniform>(), 496);
}

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

#[test]
fn environment_capture_surface_policy_is_opt_in() {
    let mut uniform = SceneUniform::default();

    assert_eq!(uniform.sky_sun_params[3], 0.0);

    uniform.use_environment_capture_surface_policy();

    assert_eq!(uniform.sky_sun_params[3], 1.0);
}
