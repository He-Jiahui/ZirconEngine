use crate::scene::viewport::{ProjectionMode, ViewportCameraSnapshot};
use zircon_runtime::core::framework::render::RenderSpatialRay;
use zircon_runtime_interface::math::{Mat4, UVec2, Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScreenProjection {
    pub(crate) position: Vec2,
    pub(crate) depth: f32,
}

pub(crate) struct ViewportProjectionContext<'a> {
    camera: &'a ViewportCameraSnapshot,
    viewport: UVec2,
    view_projection: Mat4,
}

impl<'a> ViewportProjectionContext<'a> {
    pub(crate) fn new(camera: &'a ViewportCameraSnapshot, viewport: UVec2) -> Self {
        let viewport = UVec2::new(viewport.x.max(1), viewport.y.max(1));
        let aspect = viewport.x as f32 / viewport.y as f32;
        let projection = match camera.projection_mode {
            ProjectionMode::Perspective => zircon_runtime_interface::math::perspective(
                camera.fov_y_radians,
                aspect,
                camera.z_near,
                camera.z_far,
            ),
            ProjectionMode::Orthographic => {
                let half_height = camera.ortho_size.max(0.01);
                let half_width = half_height * aspect.max(0.001);
                Mat4::orthographic_rh(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    camera.z_near.max(0.001),
                    camera.z_far,
                )
            }
        };
        Self {
            camera,
            viewport,
            view_projection: projection
                * zircon_runtime_interface::math::view_matrix(camera.transform),
        }
    }

    pub(crate) fn projected_point(&self, world: Vec3) -> Option<ScreenProjection> {
        let clip = self.view_projection * world.extend(1.0);
        if clip.w <= f32::EPSILON {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        if ndc.z < -1.0 || ndc.z > 1.0 {
            return None;
        }
        Some(ScreenProjection {
            position: Vec2::new(
                (ndc.x * 0.5 + 0.5) * self.viewport.x as f32,
                (-ndc.y * 0.5 + 0.5) * self.viewport.y as f32,
            ),
            depth: ndc.z,
        })
    }

    pub(crate) fn world_units_per_pixel(&self, origin: Vec3) -> f32 {
        match self.camera.projection_mode {
            ProjectionMode::Perspective => {
                let distance = self.camera.transform.translation.distance(origin).max(0.5);
                distance * (self.camera.fov_y_radians * 0.5).tan() / self.viewport.y as f32 * 2.0
            }
            ProjectionMode::Orthographic => {
                self.camera.ortho_size.max(0.5) * 2.0 / self.viewport.y as f32
            }
        }
    }

    pub(crate) fn spatial_ray_at(&self, point: Vec2) -> RenderSpatialRay {
        let viewport_width = self.viewport.x.max(1) as f32;
        let viewport_height = self.viewport.y.max(1) as f32;
        let ndc_x = point.x / viewport_width * 2.0 - 1.0;
        let ndc_y = 1.0 - point.y / viewport_height * 2.0;
        let transform = self.camera.transform;
        let forward = transform.forward();
        let (origin, direction) = match self.camera.projection_mode {
            ProjectionMode::Perspective => {
                let half_height = (self.camera.fov_y_radians * 0.5).tan();
                let half_width = half_height * (viewport_width / viewport_height);
                (
                    transform.translation,
                    (forward
                        + transform.right() * (ndc_x * half_width)
                        + transform.up() * (ndc_y * half_height))
                        .normalize_or_zero(),
                )
            }
            ProjectionMode::Orthographic => {
                let half_height = self.camera.ortho_size.max(0.01);
                let half_width = half_height * (viewport_width / viewport_height);
                (
                    transform.translation
                        + transform.right() * (ndc_x * half_width)
                        + transform.up() * (ndc_y * half_height),
                    forward,
                )
            }
        };
        RenderSpatialRay::new(origin, direction, self.camera.z_far.max(self.camera.z_near))
    }
}

pub(crate) fn project_point(
    world: Vec3,
    camera: &ViewportCameraSnapshot,
    viewport: UVec2,
) -> Option<Vec2> {
    ViewportProjectionContext::new(camera, viewport)
        .projected_point(world)
        .map(|projection| projection.position)
}

pub(crate) fn world_units_per_pixel(
    camera: &ViewportCameraSnapshot,
    origin: Vec3,
    viewport: UVec2,
) -> f32 {
    ViewportProjectionContext::new(camera, viewport).world_units_per_pixel(origin)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_projection_context_projects_center_and_reuses_camera_scale() {
        let camera = ViewportCameraSnapshot::default();
        let context = ViewportProjectionContext::new(&camera, UVec2::new(800, 600));

        let center = context
            .projected_point(Vec3::new(0.0, 0.0, -5.0))
            .expect("point in front of the camera should project");

        assert!((center.position.x - 400.0).abs() < 0.01);
        assert!((center.position.y - 300.0).abs() < 0.01);
        assert!(context.world_units_per_pixel(Vec3::new(0.0, 0.0, -5.0)) > 0.0);
    }

    #[test]
    fn shared_projection_context_builds_a_center_cursor_ray_from_the_camera() {
        let camera = ViewportCameraSnapshot::default();
        let context = ViewportProjectionContext::new(&camera, UVec2::new(800, 600));
        let ray = context.spatial_ray_at(Vec2::new(400.0, 300.0));

        assert_eq!(ray.origin, camera.transform.translation);
        assert!(ray.direction.dot(camera.transform.forward()) > 0.999);
        assert!(ray.max_distance >= camera.z_near);
    }

    #[test]
    fn shared_projection_context_fails_closed_for_an_invalid_perspective_ray() {
        let mut camera = ViewportCameraSnapshot::default();
        camera.fov_y_radians = f32::NAN;
        let ray = ViewportProjectionContext::new(&camera, UVec2::new(800, 600))
            .spatial_ray_at(Vec2::new(400.0, 300.0));

        assert_eq!(ray.direction, Vec3::ZERO);
    }

    #[test]
    fn pointer_candidate_pipeline_constructs_projection_context_once() {
        let root = include_str!("pointer/candidates/precision_candidates_from_layout.rs");
        let leaf_sources = [
            include_str!("pointer/candidates/handle_candidate.rs"),
            include_str!("pointer/candidates/projected_ring_segments.rs"),
            include_str!("pointer/candidates/renderable_candidate.rs"),
            include_str!("pointer/candidates/scene_gizmo_candidate.rs"),
        ];

        assert_eq!(root.matches("ViewportProjectionContext::new").count(), 1);
        assert!(
            leaf_sources
                .iter()
                .all(|source| !source.contains("ViewportProjectionContext::new"))
        );
    }
}
