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

#[test]
fn perspective_mesh_spheres_use_world_distance_to_each_side_plane() {
    // UE MakeFrustumPlane normalizes the plane before IntersectSphere compares
    // a world-space radius. Use the signed-distance oracle, not the old equations.
    let mut mismatches = Vec::new();
    for (aspect, horizontal_hit, horizontal_miss) in
        [(0.5, 3.5625, 3.75), (1.0, 6.25, 6.5), (2.0, 12.0, 12.5)]
    {
        let camera = ViewportCameraSnapshot {
            fov_y_radians: std::f32::consts::FRAC_PI_2,
            aspect_ratio: aspect,
            z_far: 10.0,
            ..Default::default()
        };
        let frustum = BoundsVisibilityTest::new(&camera);
        for (axis, slope, hit, miss) in [
            (Vec3::X, aspect, horizontal_hit, horizontal_miss),
            (Vec3::Y, 1.0, 6.25, 6.5),
        ] {
            for sign in [-1.0, 1.0] {
                let outward = sign * axis;
                let plane_normal = (outward + slope * Vec3::Z).normalize();
                for (side, expected) in [(hit, true), (miss, false)] {
                    let center = side * outward - 5.0 * Vec3::Z;
                    let signed_distance = plane_normal.dot(center);
                    assert_eq!(
                        signed_distance <= 1.0,
                        expected,
                        "fixture must lie on the intended side of the unit-radius boundary"
                    );
                    let visible = frustum.is_visible(VisibilityBounds {
                        center,
                        radius: 1.0,
                    });
                    if visible != expected {
                        mismatches.push((
                            aspect,
                            outward,
                            center,
                            signed_distance,
                            visible,
                            expected,
                        ));
                    }
                }
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "normalized side-plane sphere oracle: {mismatches:?}"
    );
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
