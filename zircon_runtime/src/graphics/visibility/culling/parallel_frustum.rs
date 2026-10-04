use crate::core::framework::render::ViewportCameraSnapshot;
use crate::core::{parallel_map_indices, TaskPool};
use crate::graphics::visibility::VisibilityBounds;

use super::is_mesh_visible::BoundsVisibilityTest;

const PARALLEL_FRUSTUM_MIN_MESHES: usize = 64;

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
    parallel_map_indices(task_pool, candidates.len(), |index| {
        let candidate = candidates[index];
        MeshFrustumVisibility {
            stable_instance_key: candidate.stable_instance_key,
            visible: visibility_test.is_visible(candidate.bounds),
        }
    })
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
#[path = "tests/parallel_frustum.rs"]
mod tests;
