use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use crate::core::math::{Real, UVec2, Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerRay {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl PointerRay {
    pub fn new(origin: Vec3, direction: Vec3) -> Option<Self> {
        let direction = direction.normalize_or_zero();
        (origin.is_finite() && direction.is_finite() && direction.length_squared() > 0.0)
            .then_some(Self { origin, direction })
    }
}

pub fn ray_from_viewport_point(
    camera: &ViewportCameraSnapshot,
    viewport_size: UVec2,
    position: Vec2,
) -> Option<PointerRay> {
    if viewport_size.x == 0 || viewport_size.y == 0 {
        return None;
    }
    if position.x < 0.0
        || position.y < 0.0
        || position.x > viewport_size.x as Real
        || position.y > viewport_size.y as Real
    {
        return None;
    }

    let ndc_x = position.x / viewport_size.x as Real * 2.0 - 1.0;
    let ndc_y = 1.0 - position.y / viewport_size.y as Real * 2.0;
    let aspect_ratio = viewport_size.x.max(1) as Real / viewport_size.y.max(1) as Real;
    match camera.projection_mode {
        ProjectionMode::Perspective => perspective_ray(camera, aspect_ratio, ndc_x, ndc_y),
        ProjectionMode::Orthographic => orthographic_ray(camera, aspect_ratio, ndc_x, ndc_y),
    }
}

fn perspective_ray(
    camera: &ViewportCameraSnapshot,
    aspect_ratio: Real,
    ndc_x: Real,
    ndc_y: Real,
) -> Option<PointerRay> {
    let half_fov_tan = (camera.fov_y_radians * 0.5).tan();
    if !half_fov_tan.is_finite() || half_fov_tan <= 0.0 {
        return None;
    }
    let local_direction = Vec3::new(
        ndc_x * aspect_ratio.max(0.001) * half_fov_tan,
        ndc_y * half_fov_tan,
        -1.0,
    );
    PointerRay::new(
        camera.transform.translation,
        camera.transform.rotation * local_direction,
    )
}

fn orthographic_ray(
    camera: &ViewportCameraSnapshot,
    aspect_ratio: Real,
    ndc_x: Real,
    ndc_y: Real,
) -> Option<PointerRay> {
    let half_height = camera.ortho_size.max(0.001);
    let half_width = half_height * aspect_ratio.max(0.001);
    let origin = camera.transform.translation
        + camera.transform.right() * (ndc_x * half_width)
        + camera.transform.up() * (ndc_y * half_height);
    PointerRay::new(origin, camera.transform.forward())
}
