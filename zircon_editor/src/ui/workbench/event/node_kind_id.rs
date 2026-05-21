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
