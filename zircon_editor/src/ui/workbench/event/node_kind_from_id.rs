use zircon_runtime::scene::components::NodeKind;

pub(super) fn node_kind_from_id(value: &str) -> Option<NodeKind> {
    match value {
        "camera" => Some(NodeKind::Camera),
        "cube" => Some(NodeKind::Cube),
        "mesh" => Some(NodeKind::Mesh),
        "ambient_light" => Some(NodeKind::AmbientLight),
        "directional_light" => Some(NodeKind::DirectionalLight),
        "point_light" => Some(NodeKind::PointLight),
        "rect_light" => Some(NodeKind::RectLight),
        "spot_light" => Some(NodeKind::SpotLight),
        _ => None,
    }
}

pub(super) fn node_kind_from_control_id(value: &str) -> Option<NodeKind> {
    match value {
        "Camera" => Some(NodeKind::Camera),
        "Cube" => Some(NodeKind::Cube),
        "Mesh" => Some(NodeKind::Mesh),
        "AmbientLight" => Some(NodeKind::AmbientLight),
        "DirectionalLight" => Some(NodeKind::DirectionalLight),
        "PointLight" => Some(NodeKind::PointLight),
        "RectLight" => Some(NodeKind::RectLight),
        "SpotLight" => Some(NodeKind::SpotLight),
        _ => None,
    }
}
