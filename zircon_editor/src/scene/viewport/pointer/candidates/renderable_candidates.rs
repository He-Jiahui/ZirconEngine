use std::collections::HashSet;

use crate::scene::viewport::RenderMeshSnapshot;

use crate::scene::viewport::pointer::viewport_renderable_pick_candidate::ViewportRenderablePickCandidate;

use super::renderable_pick_radius;

pub(in crate::scene::viewport::pointer) fn renderable_candidates(
    render_meshes: &[RenderMeshSnapshot],
) -> Vec<ViewportRenderablePickCandidate> {
    let mut candidates: Vec<ViewportRenderablePickCandidate> =
        Vec::with_capacity(render_meshes.len());
    let mut previous_owner: Option<u64> = None;
    let mut seen_owners: Option<HashSet<u64>> = None;
    for mesh in render_meshes {
        if !admit_renderable_owner(
            mesh.node_id,
            &mut previous_owner,
            &mut seen_owners,
            candidates.iter().map(|candidate| candidate.owner),
        ) {
            continue;
        }
        candidates.push(ViewportRenderablePickCandidate {
            owner: mesh.node_id,
            position: mesh.transform.translation,
            radius_world: renderable_pick_radius(mesh.transform),
        });
    }
    candidates
}

fn admit_renderable_owner(
    owner: u64,
    previous_owner: &mut Option<u64>,
    seen_owners: &mut Option<HashSet<u64>>,
    admitted_owners: impl Iterator<Item = u64>,
) -> bool {
    if *previous_owner == Some(owner) {
        return false;
    }
    if seen_owners.is_none()
        && previous_owner
            .as_ref()
            .is_some_and(|previous| owner < *previous)
    {
        *seen_owners = Some(admitted_owners.collect::<HashSet<_>>());
    }
    *previous_owner = Some(owner);
    match seen_owners.as_mut() {
        Some(seen_owners) => seen_owners.insert(owner),
        None => true,
    }
}

#[cfg(test)]
#[path = "tests/renderable_candidates.rs"]
mod tests;
