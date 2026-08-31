use std::collections::HashSet;

use crate::scene::viewport::RenderMeshSnapshot;

use crate::scene::viewport::pointer::viewport_renderable_pick_candidate::ViewportRenderablePickCandidate;

use super::renderable_pick_radius;

pub(in crate::scene::viewport::pointer) fn renderable_candidates(
    render_meshes: &[RenderMeshSnapshot],
) -> Vec<ViewportRenderablePickCandidate> {
    let mut candidates = Vec::with_capacity(render_meshes.len());
    let mut previous_owner = None;
    let mut seen_owners = None;
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
mod tests {
    use std::collections::HashSet;

    use super::admit_renderable_owner;

    fn admitted_owners(owners: &[u64]) -> (Vec<u64>, Option<HashSet<u64>>) {
        let mut admitted = Vec::with_capacity(owners.len());
        let mut previous_owner = None;
        let mut seen_owners = None;
        for &owner in owners {
            if admit_renderable_owner(
                owner,
                &mut previous_owner,
                &mut seen_owners,
                admitted.iter().copied(),
            ) {
                admitted.push(owner);
            }
        }
        (admitted, seen_owners)
    }

    #[test]
    fn grouped_owner_sequence_keeps_lazy_index_unallocated() {
        let (owners, seen_owners) = admitted_owners(&[1, 1, 2, 2, 3, 3]);

        assert_eq!(owners, [1, 2, 3]);
        assert!(seen_owners.is_none());
    }

    #[test]
    fn interleaved_owner_sequence_collapses_non_adjacent_duplicates() {
        let (owners, seen_owners) = admitted_owners(&[1, 2, 3, 1, 2, 3]);

        assert_eq!(owners, [1, 2, 3]);
        assert!(seen_owners.is_some());
    }
}
