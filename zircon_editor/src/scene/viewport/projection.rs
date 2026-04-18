use zircon_math::{UVec2, Vec2, Vec3};
use zircon_scene::{ProjectionMode, ViewportCameraSnapshot};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScreenProjection {
    pub(crate) position: Vec2,
    pub(crate) depth: f32,
}

pub(crate) fn projected_point(
    world: Vec3,
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
) -> Option<ScreenProjection> {
    let viewport = UVec2::new(viewport.x.max(1), viewport.y.max(1));
    let aspect = viewport.x as f32 / viewport.y.max(1) as f32;
    let projection = match camera.projection_mode {
        ProjectionMode::Perspective => {
            zircon_math::perspective(camera.fov_y_radians, aspect, camera.z_near, camera.z_far)
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let half_width = half_height * aspect.max(0.001);
            zircon_math::Mat4::orthographic_rh(
                -half_width,
                half_width,
                -half_height,
                half_height,
                camera.z_near.max(0.001),
                camera.z_far,
            )
        }
    };
    let clip = projection * zircon_math::view_matrix(camera.transform) * world.extend(1.0);
    if clip.w <= f32::EPSILON {
        return None;
    }
    let ndc = clip.truncate() / clip.w;
    if ndc.z < -1.0 || ndc.z > 1.0 {
        return None;
    }
    Some(ScreenProjection {
        position: Vec2::new(
            (ndc.x * 0.5 + 0.5) * viewport.x as f32,
            (-ndc.y * 0.5 + 0.5) * viewport.y as f32,
        ),
        depth: ndc.z,
    })
}

pub(crate) fn project_point(
    world: Vec3,
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
) -> Option<Vec2> {
    projected_point(world, camera, viewport).map(|projection| projection.position)
}

pub(crate) fn world_units_per_pixel(
    camera: &ViewportCameraSnapshot,
    origin: Vec3,
    viewport: UVec2,
) -> f32 {
    match camera.projection_mode {
        ProjectionMode::Perspective => {
            let distance = camera.transform.translation.distance(origin).max(0.5);
            distance * (camera.fov_y_radians * 0.5).tan() / viewport.y.max(1) as f32 * 2.0
        }
        ProjectionMode::Orthographic => camera.ortho_size.max(0.5) * 2.0 / viewport.y.max(1) as f32,
    }
}

pub(crate) fn distance_to_segment(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_sq = segment.length_squared();
    if length_sq <= f32::EPSILON {
        return point.distance(start);
    }
    let t = ((point - start).dot(segment) / length_sq).clamp(0.0, 1.0);
    point.distance(start + segment * t)
}
