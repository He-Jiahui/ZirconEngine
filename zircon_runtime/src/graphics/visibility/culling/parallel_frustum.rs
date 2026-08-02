use crate::core::framework::render::ViewportCameraSnapshot;
use crate::core::{parallel_for, TaskPool};
use crate::graphics::visibility::VisibilityBounds;

use super::is_mesh_visible::BoundsVisibilityTest;

const PARALLEL_FRUSTUM_MIN_MESHES: usize = 64;
const PARALLEL_FRUSTUM_CHUNK_SIZE: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeshFrustumCandidate {
    pub(crate) stable_instance_key: u64,
    pub(crate) bounds: VisibilityBounds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MeshFrustumVisibility {
    pub(crate) stable_instance_key: u64,
    pub(crate) visible: bool,
}

pub(crate) fn mesh_frustum_visibility(
    candidates: &[MeshFrustumCandidate],
    camera: &ViewportCameraSnapshot,
    task_pool: Option<&TaskPool>,
) -> Vec<MeshFrustumVisibility> {
    let Some(task_pool) = task_pool else {
        return serial_mesh_frustum_visibility(candidates, camera);
    };

    if candidates.len() < PARALLEL_FRUSTUM_MIN_MESHES {
        return serial_mesh_frustum_visibility(candidates, camera);
    }

    let visibility_test = BoundsVisibilityTest::new(camera);
    let mut work_items = candidates
        .iter()
        .map(|candidate| MeshFrustumWorkItem {
            candidate: *candidate,
            visible: false,
        })
        .collect::<Vec<_>>();

    parallel_for(
        task_pool,
        &mut work_items,
        PARALLEL_FRUSTUM_CHUNK_SIZE,
        |chunk| {
            for item in chunk {
                item.visible = visibility_test.is_visible(item.candidate.bounds);
            }
        },
    );

    work_items
        .into_iter()
        .map(|item| MeshFrustumVisibility {
            stable_instance_key: item.candidate.stable_instance_key,
            visible: item.visible,
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
struct MeshFrustumWorkItem {
    candidate: MeshFrustumCandidate,
    visible: bool,
}

pub(crate) fn serial_mesh_frustum_visibility(
    candidates: &[MeshFrustumCandidate],
    camera: &ViewportCameraSnapshot,
) -> Vec<MeshFrustumVisibility> {
    let visibility_test = BoundsVisibilityTest::new(camera);
    candidates
        .iter()
        .map(|candidate| MeshFrustumVisibility {
            stable_instance_key: candidate.stable_instance_key,
            visible: visibility_test.is_visible(candidate.bounds),
        })
        .collect()
}

#[cfg(test)]
mod tests {
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
        let source = include_str!("parallel_frustum.rs");
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
}
