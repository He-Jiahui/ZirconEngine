use zircon_runtime::scene::components::NodeKind;

pub(super) fn node_kind_id(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Camera => "Camera",
        NodeKind::Cube => "Cube",
        NodeKind::Mesh => "Mesh",
        NodeKind::AmbientLight => "AmbientLight",
        NodeKind::DirectionalLight => "DirectionalLight",
        NodeKind::PointLight => "PointLight",
        NodeKind::RectLight => "RectLight",
        NodeKind::SpotLight => "SpotLight",
    }
}

pub(super) fn node_kind_action_id(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Camera => "camera",
        NodeKind::Cube => "cube",
        NodeKind::Mesh => "mesh",
        NodeKind::AmbientLight => "ambient_light",
        NodeKind::DirectionalLight => "directional_light",
        NodeKind::PointLight => "point_light",
        NodeKind::RectLight => "rect_light",
        NodeKind::SpotLight => "spot_light",
    }
}
