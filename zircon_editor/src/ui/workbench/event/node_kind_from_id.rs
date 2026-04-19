use zircon_scene::components::NodeKind;

pub(super) fn node_kind_from_id(value: &str) -> Option<NodeKind> {
    match value {
        "Camera" => Some(NodeKind::Camera),
        "Cube" => Some(NodeKind::Cube),
        "Mesh" => Some(NodeKind::Mesh),
        "DirectionalLight" => Some(NodeKind::DirectionalLight),
        _ => None,
    }
}
