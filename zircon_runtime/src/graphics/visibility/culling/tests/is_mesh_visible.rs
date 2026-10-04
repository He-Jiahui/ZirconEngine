use super::BoundsVisibilityTest;
use crate::core::framework::render::{ProjectionMode, ViewportCameraSnapshot};
use crate::core::math::{view_matrix, Vec3};
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
