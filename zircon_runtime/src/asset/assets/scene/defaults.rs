use crate::core::framework::render::{DEFAULT_CAMERA_EXPOSURE_EV100, DEFAULT_CAMERA_MSAA_SAMPLES};
use crate::core::math::Real;

pub(super) fn default_camera_fov_y_radians() -> Real {
    60.0_f32.to_radians()
}

pub(super) const fn default_camera_ortho_size() -> Real {
    5.0
}

pub(super) const fn default_camera_z_near() -> Real {
    0.1
}

pub(super) const fn default_camera_z_far() -> Real {
    200.0
}

pub(super) const fn default_camera_exposure_ev100() -> Real {
    DEFAULT_CAMERA_EXPOSURE_EV100
}

pub(super) const fn default_camera_msaa_samples() -> u32 {
    DEFAULT_CAMERA_MSAA_SAMPLES
}

pub(super) const fn default_viewport_depth_max() -> Real {
    1.0
}

pub(super) const fn default_bloom_threshold() -> Real {
    1.0
}

pub(super) const fn default_one_real() -> Real {
    1.0
}

pub(super) const fn default_vignette_smoothness() -> Real {
    0.5
}

pub(super) const fn default_color_white() -> [Real; 3] {
    [1.0, 1.0, 1.0]
}

pub(super) fn is_zero_i32(value: &i32) -> bool {
    *value == 0
}

pub(super) fn is_zero_real(value: &Real) -> bool {
    *value == 0.0
}

pub(super) const fn default_collision_mask() -> u32 {
    u32::MAX
}

pub(super) const fn default_rigid_body_mass() -> Real {
    1.0
}

pub(super) const fn default_gravity_scale() -> Real {
    1.0
}

pub(super) const fn default_playback_speed() -> Real {
    1.0
}

pub(super) const fn default_animation_weight() -> Real {
    1.0
}

pub(super) const fn default_true() -> bool {
    true
}

pub(super) const fn default_vec3_zero() -> [Real; 3] {
    [0.0, 0.0, 0.0]
}

pub(super) const fn default_vec3_up() -> [Real; 3] {
    [0.0, 1.0, 0.0]
}

pub(super) const fn default_light_color() -> [Real; 3] {
    [1.0, 1.0, 1.0]
}

pub(super) const fn default_ambient_light_intensity() -> Real {
    80.0
}

pub(super) const fn default_rect_light_intensity() -> Real {
    1_000_000.0
}

pub(super) const fn default_rect_light_range() -> Real {
    20.0
}

pub(super) const fn default_rect_light_size() -> [Real; 2] {
    [1.0, 1.0]
}

pub(super) const fn default_scene_active() -> bool {
    true
}

pub(super) const fn default_render_layer_mask() -> u32 {
    0x0000_0001
}
