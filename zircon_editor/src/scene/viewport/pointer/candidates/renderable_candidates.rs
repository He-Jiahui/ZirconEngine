use crate::scene::viewport::RenderMeshSnapshot;

use crate::scene::viewport::pointer::viewport_renderable_pick_candidate::ViewportRenderablePickCandidate;

use super::renderable_pick_radius;

pub(in crate::scene::viewport::pointer) fn renderable_candidates(
    render_meshes: &[RenderMeshSnapshot],
) -> Vec<ViewportRenderablePickCandidate> {
    let mut candidates = Vec::with_capacity(render_meshes.len());
    let mut previous_owner = None;
    for mesh in render_meshes {
        if previous_owner == Some(mesh.node_id) {
            continue;
        }
        previous_owner = Some(mesh.node_id);
        candidates.push(ViewportRenderablePickCandidate {
            owner: mesh.node_id,
            position: mesh.transform.translation,
            radius_world: renderable_pick_radius(mesh.transform),
        });
    }
    candidates
}
