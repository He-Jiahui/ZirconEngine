use zircon_scene::Scene;

pub(in crate::core::editing::state) fn hierarchy_depth(
    scene: &Scene,
    node_id: zircon_scene::NodeId,
) -> usize {
    let mut depth = 0;
    let mut cursor = scene.find_node(node_id).and_then(|node| node.parent);
    while let Some(parent) = cursor {
        depth += 1;
        cursor = scene.find_node(parent).and_then(|node| node.parent);
    }
    depth
}
