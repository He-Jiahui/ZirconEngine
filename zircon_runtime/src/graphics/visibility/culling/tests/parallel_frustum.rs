use crate::core::math::Vec3;
use crate::core::{TaskPool, TaskPoolDescriptor};
use crate::graphics::visibility::VisibilityBounds;

use super::{mesh_frustum_visibility, serial_mesh_frustum_visibility, MeshFrustumCandidate};

#[test]
fn parallel_frustum_visibility_matches_serial_order_and_results() {
    let camera = crate::core::framework::render::ViewportCameraSnapshot::default();
    let candidates = (0..96)
        .map(|index| {
            let z = if index % 3 == 0 { 5.0 } else { -5.0 };
            candidate_at(index + 1, Vec3::new(index as f32 * 0.01, 0.0, z))
        })
        .collect::<Vec<_>>();

    let serial = serial_mesh_frustum_visibility(&candidates, &camera);
    let task_pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let parallel = mesh_frustum_visibility(&candidates, &camera, Some(&task_pool));

    assert_eq!(parallel, serial);
    assert!(parallel.iter().any(|entry| entry.visible));
    assert!(parallel.iter().any(|entry| !entry.visible));
    assert_eq!(
        parallel
            .iter()
            .map(|entry| entry.stable_instance_key)
            .collect::<Vec<_>>(),
        (1..=96).collect::<Vec<_>>()
    );
}

#[test]
fn frustum_visibility_precomputes_camera_test_once_per_path() {
    let source = include_str!("../parallel_frustum.rs");
    let constructor = concat!("BoundsVisibility", "Test::new(camera)");

    assert_eq!(source.matches(constructor).count(), 2);
    assert!(!source.contains(concat!("is_bounds_visible(", "item.candidate.bounds")));
}

fn candidate_at(stable_instance_key: u64, center: Vec3) -> MeshFrustumCandidate {
    MeshFrustumCandidate {
        stable_instance_key,
        bounds: VisibilityBounds {
            center,
            radius: 0.5,
        },
    }
}
