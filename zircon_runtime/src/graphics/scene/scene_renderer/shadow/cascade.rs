use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use crate::core::math::{is_finite_vec3, Mat4, Real, Vec3};

pub(crate) const MAX_SHADOW_CASCADES: usize = 4;
pub(crate) const DEFAULT_CASCADE_COUNT: u32 = 4;
pub(crate) const DEFAULT_CASCADE_MAX_DISTANCE: Real = 150.0;
pub(crate) const DEFAULT_CASCADE_LOG_LINEAR_LAMBDA: Real = 0.7;
pub(crate) const DEFAULT_CASCADE_FADE_FRACTION: Real = 0.1;
const MIN_CASCADE_NEAR_PLANE: Real = 0.001;
const MIN_CASCADE_DEPTH_RANGE: Real = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CascadeSplitConfig {
    pub(crate) cascade_count: u32,
    pub(crate) max_distance: Real,
    pub(crate) log_linear_lambda: Real,
    pub(crate) fade_fraction: Real,
}

impl CascadeSplitConfig {
    pub(crate) fn effective_cascade_count(self) -> usize {
        self.cascade_count.clamp(1, MAX_SHADOW_CASCADES as u32) as usize
    }

    fn sanitized(self, near: Real) -> Self {
        let near = sanitize_positive_distance(near, MIN_CASCADE_NEAR_PLANE);
        Self {
            cascade_count: self.cascade_count.clamp(1, MAX_SHADOW_CASCADES as u32),
            max_distance: sanitize_positive_distance(
                self.max_distance.max(near + MIN_CASCADE_DEPTH_RANGE),
                near + MIN_CASCADE_DEPTH_RANGE,
            ),
            log_linear_lambda: sanitize_unit_interval(self.log_linear_lambda),
            fade_fraction: sanitize_unit_interval(self.fade_fraction).min(0.5),
        }
    }
}

impl Default for CascadeSplitConfig {
    fn default() -> Self {
        Self {
            cascade_count: DEFAULT_CASCADE_COUNT,
            max_distance: DEFAULT_CASCADE_MAX_DISTANCE,
            log_linear_lambda: DEFAULT_CASCADE_LOG_LINEAR_LAMBDA,
            fade_fraction: DEFAULT_CASCADE_FADE_FRACTION,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct CascadeRange {
    pub(crate) index: u32,
    pub(crate) near: Real,
    pub(crate) far: Real,
    pub(crate) fade_start: Real,
    pub(crate) fade_length: Real,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CascadeShadowBounds {
    pub(crate) center: Vec3,
    pub(crate) radius: Real,
    pub(crate) near_plane: Real,
    pub(crate) far_plane: Real,
}

impl CascadeShadowBounds {
    pub(crate) fn new(center: Vec3, radius: Real) -> Self {
        let radius = stabilized_cascade_radius(radius);
        Self {
            center,
            radius,
            near_plane: MIN_CASCADE_NEAR_PLANE,
            far_plane: (radius * 4.0).max(MIN_CASCADE_DEPTH_RANGE),
        }
    }

    pub(crate) fn with_depth_range(mut self, near_plane: Real, far_plane: Real) -> Self {
        self.near_plane = sanitize_positive_distance(near_plane, MIN_CASCADE_NEAR_PLANE);
        self.far_plane = sanitize_positive_distance(
            far_plane.max(self.near_plane + MIN_CASCADE_DEPTH_RANGE),
            self.near_plane + MIN_CASCADE_DEPTH_RANGE,
        );
        self
    }
}

/// 在线性和对数深度分布之间按 lambda 插值；调用者用相邻 split 形成级联区间，并在末端加入淡出带以衔接级联。
pub(crate) fn compute_cascade_splits(
    config: &CascadeSplitConfig,
    near: Real,
) -> [Real; MAX_SHADOW_CASCADES + 1] {
    let near = sanitize_positive_distance(near, MIN_CASCADE_NEAR_PLANE);
    let config = config.sanitized(near);
    let cascade_count = config.effective_cascade_count();
    let far = config.max_distance;
    let lambda = config.log_linear_lambda;
    let mut splits = [far; MAX_SHADOW_CASCADES + 1];
    splits[0] = near;

    for cascade_index in 1..=cascade_count {
        let factor = cascade_index as Real / cascade_count as Real;
        let linear = near + (far - near) * factor;
        let logarithmic = near * (far / near).powf(factor);
        splits[cascade_index] = linear + (logarithmic - linear) * lambda;
    }

    splits
}

pub(crate) fn compute_cascade_ranges(config: &CascadeSplitConfig, near: Real) -> Vec<CascadeRange> {
    let near = sanitize_positive_distance(near, MIN_CASCADE_NEAR_PLANE);
    let config = config.sanitized(near);
    let splits = compute_cascade_splits(&config, near);
    let cascade_count = config.effective_cascade_count();
    (0..cascade_count)
        .map(|index| {
            let range_near = splits[index];
            let range_far = splits[index + 1].max(range_near + MIN_CASCADE_NEAR_PLANE);
            let fade_length =
                ((range_far - range_near) * config.fade_fraction).min(range_far - range_near);
            CascadeRange {
                index: index as u32,
                near: range_near,
                far: range_far,
                fade_start: range_far - fade_length,
                fade_length,
            }
        })
        .collect()
}

pub(crate) fn stabilized_cascade_radius(radius: Real) -> Real {
    if radius.is_finite() && radius > 0.0 {
        (radius * 100.0).ceil() / 100.0
    } else {
        MIN_CASCADE_DEPTH_RANGE
    }
}

pub(crate) fn cascade_world_units_per_texel(radius: Real, resolution: u32) -> Real {
    let radius = stabilized_cascade_radius(radius);
    let resolution = resolution.max(1) as Real;
    (radius * 2.0) / resolution
}

pub(crate) fn cascade_shadow_bounds_from_camera_slice(
    camera: &ViewportCameraSnapshot,
    range: CascadeRange,
) -> CascadeShadowBounds {
    let (near, far) = camera_slice_distances(camera, range);
    let corners = camera_slice_corners(camera, near, far);
    let center = corners.iter().copied().sum::<Vec3>() / corners.len() as Real;
    let radius = corners
        .iter()
        .fold(0.0, |radius: Real, corner| {
            radius.max((*corner - center).length())
        })
        .max(MIN_CASCADE_DEPTH_RANGE);

    CascadeShadowBounds::new(center, radius)
}

pub(crate) fn snap_light_space_center_to_texel(
    light_view: Mat4,
    center: Vec3,
    world_units_per_texel: Real,
) -> Vec3 {
    let texel = sanitize_positive_distance(world_units_per_texel, MIN_CASCADE_NEAR_PLANE);
    let light_space_center = light_view.transform_point3(center);
    Vec3::new(
        (light_space_center.x / texel).floor() * texel,
        (light_space_center.y / texel).floor() * texel,
        light_space_center.z,
    )
}

/// 将级联中心的光空间 x/y 量化到 texel 网格；光视图、半径与深度范围固定且中心未越过网格边界时，可复用同一正交投影。
pub(crate) fn snapped_cascade_view_projection(
    light_view: Mat4,
    bounds: CascadeShadowBounds,
    resolution: u32,
) -> Mat4 {
    let radius = stabilized_cascade_radius(bounds.radius);
    let texel = cascade_world_units_per_texel(radius, resolution);
    let center = snap_light_space_center_to_texel(light_view, bounds.center, texel);
    let near_plane = sanitize_positive_distance(bounds.near_plane, MIN_CASCADE_NEAR_PLANE);
    let far_plane = sanitize_positive_distance(
        bounds.far_plane.max(near_plane + MIN_CASCADE_DEPTH_RANGE),
        near_plane + MIN_CASCADE_DEPTH_RANGE,
    );
    let projection = Mat4::orthographic_rh(
        center.x - radius,
        center.x + radius,
        center.y - radius,
        center.y + radius,
        near_plane,
        far_plane,
    );
    projection * light_view
}

fn sanitize_positive_distance(value: Real, fallback: Real) -> Real {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn camera_slice_distances(camera: &ViewportCameraSnapshot, range: CascadeRange) -> (Real, Real) {
    let camera_near = sanitize_positive_distance(camera.z_near, MIN_CASCADE_NEAR_PLANE);
    let camera_far = sanitize_positive_distance(
        camera.z_far.max(camera_near + MIN_CASCADE_DEPTH_RANGE),
        camera_near + MIN_CASCADE_DEPTH_RANGE,
    );
    let near = sanitize_positive_distance(range.near.max(camera_near), camera_near)
        .min((camera_far - MIN_CASCADE_NEAR_PLANE).max(camera_near));
    let far = sanitize_positive_distance(
        range.far.max(near + MIN_CASCADE_NEAR_PLANE),
        near + MIN_CASCADE_DEPTH_RANGE,
    )
    .min(camera_far)
    .max(near + MIN_CASCADE_NEAR_PLANE);

    (near, far)
}

fn camera_slice_corners(camera: &ViewportCameraSnapshot, near: Real, far: Real) -> [Vec3; 8] {
    let origin = finite_vec3_or(camera.transform.translation, Vec3::ZERO);
    let forward = finite_direction_or(camera.transform.forward(), -Vec3::Z);
    let right = finite_direction_or(camera.transform.right(), Vec3::X);
    let up = finite_direction_or(camera.transform.up(), Vec3::Y);
    let (near_half_width, near_half_height, far_half_width, far_half_height) =
        camera_slice_half_extents(camera, near, far);
    let near_center = origin + forward * near;
    let far_center = origin + forward * far;

    [
        near_center + up * near_half_height - right * near_half_width,
        near_center + up * near_half_height + right * near_half_width,
        near_center - up * near_half_height - right * near_half_width,
        near_center - up * near_half_height + right * near_half_width,
        far_center + up * far_half_height - right * far_half_width,
        far_center + up * far_half_height + right * far_half_width,
        far_center - up * far_half_height - right * far_half_width,
        far_center - up * far_half_height + right * far_half_width,
    ]
}

fn camera_slice_half_extents(
    camera: &ViewportCameraSnapshot,
    near: Real,
    far: Real,
) -> (Real, Real, Real, Real) {
    let aspect = sanitize_positive_distance(camera.aspect_ratio.abs(), 1.0);
    match camera.projection_mode {
        ProjectionMode::Perspective => {
            let half_fov_tan = sanitize_half_fov_tangent(camera.fov_y_radians);
            let near_half_height = near * half_fov_tan;
            let far_half_height = far * half_fov_tan;
            (
                near_half_height * aspect,
                near_half_height,
                far_half_height * aspect,
                far_half_height,
            )
        }
        ProjectionMode::Orthographic => {
            let half_height =
                sanitize_positive_distance(camera.ortho_size.abs(), MIN_CASCADE_DEPTH_RANGE);
            let half_width = half_height * aspect;
            (half_width, half_height, half_width, half_height)
        }
    }
}

fn sanitize_half_fov_tangent(fov_y_radians: Real) -> Real {
    if fov_y_radians.is_finite() {
        (fov_y_radians.clamp(0.001, std::f32::consts::PI - 0.001) * 0.5)
            .tan()
            .max(0.001)
    } else {
        (60.0_f32.to_radians() * 0.5).tan()
    }
}

fn finite_direction_or(value: Vec3, fallback: Vec3) -> Vec3 {
    if is_finite_vec3(value) && value.length_squared() > f32::EPSILON {
        value.normalize_or_zero()
    } else {
        fallback
    }
}

fn finite_vec3_or(value: Vec3, fallback: Vec3) -> Vec3 {
    if is_finite_vec3(value) {
        value
    } else {
        fallback
    }
}

fn sanitize_unit_interval(value: Real) -> Real {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tests/cascade.rs"]
mod tests;
