use zircon_scene::Scene;

use crate::scene::viewport::pointer::viewport_renderable_pick_candidate::ViewportRenderablePickCandidate;

use super::renderable_pick_radius;

pub(in crate::scene::viewport::pointer) fn renderable_candidates(
    scene: &Scene,
) -> Vec<ViewportRenderablePickCandidate> {
    scene
        .nodes()
        .iter()
        .filter(|node| node.mesh.is_some() && scene.active_in_hierarchy(node.id) == Some(true))
        .map(|node| {
            let transform = scene.world_transform(node.id).unwrap_or(node.transform);
            ViewportRenderablePickCandidate {
                owner: node.id,
                position: transform.translation,
                radius_world: renderable_pick_radius(transform),
            }
        })
        .collect()
}
