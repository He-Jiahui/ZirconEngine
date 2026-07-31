use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use crate::core::math::{Mat4, Real, view_matrix};

use crate::graphics::visibility::VisibilityBounds;

#[derive(Clone, Copy, Debug)]
pub(crate) struct BoundsVisibilityTest {
    view_from_world: Mat4,
    near: Real,
    far: Real,
    projection: BoundsProjection,
}

#[derive(Clone, Copy, Debug)]
enum BoundsProjection {
    Perspective {
        half_fov_tangent: Real,
        aspect_ratio: Real,
    },
    Orthographic {
        half_width: Real,
        half_height: Real,
    },
}

impl BoundsVisibilityTest {
    pub(crate) fn new(camera: &ViewportCameraSnapshot) -> Self {
        let near = camera.z_near.max(0.001);
        let far = camera.z_far.max(near);
        let projection = match camera.projection_mode {
            ProjectionMode::Perspective => BoundsProjection::Perspective {
                half_fov_tangent: (camera.fov_y_radians * 0.5).tan(),
                aspect_ratio: camera.aspect_ratio.max(0.001),
            },
            ProjectionMode::Orthographic => {
                let half_height = camera.ortho_size.max(0.01);
                BoundsProjection::Orthographic {
                    half_width: half_height * camera.aspect_ratio.max(0.001),
                    half_height,
                }
            }
        };

        Self {
            view_from_world: view_matrix(camera.transform),
            near,
            far,
            projection,
        }
    }

    pub(crate) fn is_visible(&self, bounds: VisibilityBounds) -> bool {
        let world_radius = bounds.radius;
        let view_position = self.view_from_world.transform_point3(bounds.center);
        let depth = -view_position.z;

        if depth + world_radius < self.near || depth - world_radius > self.far {
            return false;
        }

        match self.projection {
            BoundsProjection::Perspective {
                half_fov_tangent,
                aspect_ratio,
            } => {
                let half_height = depth.max(self.near) * half_fov_tangent;
                let half_width = half_height * aspect_ratio;
                view_position.x.abs() <= half_width + world_radius
                    && view_position.y.abs() <= half_height + world_radius
            }
            BoundsProjection::Orthographic {
                half_width,
                half_height,
            } => {
                view_position.x.abs() <= half_width + world_radius
                    && view_position.y.abs() <= half_height + world_radius
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoundsVisibilityTest;
    use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
    use crate::core::math::{Vec3, view_matrix};
    use crate::graphics::visibility::VisibilityBounds;

    #[test]
    fn precomputed_bounds_test_matches_legacy_projection_equations() {
        let mut orthographic = ViewportCameraSnapshot::default();
        orthographic.projection_mode = ProjectionMode::Orthographic;
        orthographic.ortho_size = 4.0;
        let cameras = [ViewportCameraSnapshot::default(), orthographic];
        let bounds = [
            VisibilityBounds {
                center: Vec3::new(0.0, 0.0, -5.0),
                radius: 0.5,
            },
            VisibilityBounds {
                center: Vec3::new(100.0, 0.0, -5.0),
                radius: 0.5,
            },
            VisibilityBounds {
                center: Vec3::new(0.0, 0.0, 5.0),
                radius: 0.5,
            },
        ];

        for camera in &cameras {
            let visibility_test = BoundsVisibilityTest::new(camera);
            for bounds in bounds {
                assert_eq!(
                    visibility_test.is_visible(bounds),
                    legacy_bounds_visible(bounds, camera)
                );
            }
        }
    }

    fn legacy_bounds_visible(bounds: VisibilityBounds, camera: &ViewportCameraSnapshot) -> bool {
        let view_position = view_matrix(camera.transform).transform_point3(bounds.center);
        let depth = -view_position.z;
        let near = camera.z_near.max(0.001);
        let far = camera.z_far.max(near);
        if depth + bounds.radius < near || depth - bounds.radius > far {
            return false;
        }

        match camera.projection_mode {
            ProjectionMode::Perspective => {
                let half_height = depth.max(near) * (camera.fov_y_radians * 0.5).tan();
                let half_width = half_height * camera.aspect_ratio.max(0.001);
                view_position.x.abs() <= half_width + bounds.radius
                    && view_position.y.abs() <= half_height + bounds.radius
            }
            ProjectionMode::Orthographic => {
                let half_height = camera.ortho_size.max(0.01);
                let half_width = half_height * camera.aspect_ratio.max(0.001);
                view_position.x.abs() <= half_width + bounds.radius
                    && view_position.y.abs() <= half_height + bounds.radius
            }
        }
    }
}
