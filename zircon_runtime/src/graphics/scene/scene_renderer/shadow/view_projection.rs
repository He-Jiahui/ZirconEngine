use std::f32::consts::{FRAC_PI_2, PI};

use crate::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderPointLightSnapshot, RenderSpotLightSnapshot,
    ViewportCameraSnapshot,
};
use crate::core::math::{is_finite_vec3, view_matrix, Mat4, Real, Transform, Vec3};

use super::cascade::{
    cascade_shadow_bounds_from_camera_slice, snapped_cascade_view_projection, CascadeRange,
    CascadeShadowBounds,
};

const POINT_LIGHT_SHADOW_FACE_COUNT: u8 = 6;
const MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT: Real = 4.0;
const SHADOW_CAMERA_DISTANCE_SCALE: Real = 2.0;
const SHADOW_CAMERA_FAR_PADDING: Real = 64.0;
const SHADOW_CAMERA_NEAR_PLANE: Real = 0.1;
const SHADOW_CAMERA_MIN_FAR_PLANE: Real = 1.0;
const SHADOW_UP_ALIGNMENT_LIMIT: Real = 0.95;
const DEFAULT_SHADOW_LIGHT_DIRECTION_COMPONENTS: [Real; 3] = [-0.4, -1.0, -0.25];
const MIN_PUNCTUAL_SHADOW_RANGE: Real = SHADOW_CAMERA_NEAR_PLANE + SHADOW_CAMERA_MIN_FAR_PLANE;

pub(super) fn directional_cascade_view_projection(
    light: &RenderDirectionalLightSnapshot,
    camera: &ViewportCameraSnapshot,
    resolution: u32,
    range: CascadeRange,
) -> Mat4 {
    let direction = sanitize_direction(light.direction);
    let slice_bounds = cascade_shadow_bounds_from_camera_slice(camera, range);
    let half_extent = slice_bounds.radius.max(MIN_SHADOW_ORTHOGRAPHIC_HALF_EXTENT);
    let distance = half_extent * SHADOW_CAMERA_DISTANCE_SCALE + SHADOW_CAMERA_FAR_PADDING;
    let eye = slice_bounds.center - direction * distance;
    let transform = Transform::looking_at(eye, slice_bounds.center, stable_shadow_up(direction));
    let light_view = view_matrix(transform);
    let far_plane = (distance + half_extent + SHADOW_CAMERA_FAR_PADDING)
        .max(SHADOW_CAMERA_NEAR_PLANE + SHADOW_CAMERA_MIN_FAR_PLANE)
        .max(range.far);
    let bounds = CascadeShadowBounds::new(slice_bounds.center, half_extent)
        .with_depth_range(SHADOW_CAMERA_NEAR_PLANE, far_plane);
    snapped_cascade_view_projection(light_view, bounds, resolution)
}

pub(super) fn spot_light_view_projection(light: &RenderSpotLightSnapshot) -> Mat4 {
    let direction = sanitize_direction(light.direction);
    let position = finite_vec3_or(light.position, Vec3::ZERO);
    let target = position + direction;
    let view = view_matrix(Transform::looking_at(
        position,
        target,
        stable_shadow_up(direction),
    ));
    let fov_y = (light.outer_angle_radians.max(0.001) * 2.0).clamp(0.001, PI - 0.001);
    let far = sanitize_shadow_far_plane(light.range);
    Mat4::perspective_rh(fov_y, 1.0, SHADOW_CAMERA_NEAR_PLANE, far) * view
}

pub(super) fn point_light_face_view_projection(
    light: &RenderPointLightSnapshot,
    face_index: u8,
) -> Mat4 {
    let position = finite_vec3_or(light.position, Vec3::ZERO);
    let (direction, up) = point_light_face_axes(face_index);
    let view = view_matrix(Transform::looking_at(position, position + direction, up));
    let far = sanitize_shadow_far_plane(light.range);
    Mat4::perspective_rh(FRAC_PI_2, 1.0, SHADOW_CAMERA_NEAR_PLANE, far) * view
}

fn point_light_face_axes(face_index: u8) -> (Vec3, Vec3) {
    match face_index % POINT_LIGHT_SHADOW_FACE_COUNT {
        0 => (Vec3::X, Vec3::Y),
        1 => (-Vec3::X, Vec3::Y),
        2 => (Vec3::Y, Vec3::Z),
        3 => (-Vec3::Y, -Vec3::Z),
        4 => (Vec3::Z, Vec3::Y),
        _ => (-Vec3::Z, Vec3::Y),
    }
}

fn sanitize_direction(direction: Vec3) -> Vec3 {
    if is_finite_vec3(direction) && direction.length_squared() > f32::EPSILON {
        direction.normalize_or_zero()
    } else {
        default_shadow_light_direction()
    }
}

fn stable_shadow_up(direction: Vec3) -> Vec3 {
    if direction.dot(Vec3::Y).abs() > SHADOW_UP_ALIGNMENT_LIMIT {
        Vec3::X
    } else {
        Vec3::Y
    }
}

fn finite_vec3_or(value: Vec3, fallback: Vec3) -> Vec3 {
    if is_finite_vec3(value) {
        value
    } else {
        fallback
    }
}

fn sanitize_positive_distance(value: Real, fallback: Real) -> Real {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn sanitize_shadow_far_plane(value: Real) -> Real {
    sanitize_positive_distance(value, MIN_PUNCTUAL_SHADOW_RANGE).max(MIN_PUNCTUAL_SHADOW_RANGE)
}

fn default_shadow_light_direction() -> Vec3 {
    Vec3::from_array(DEFAULT_SHADOW_LIGHT_DIRECTION_COMPONENTS).normalize_or_zero()
}
