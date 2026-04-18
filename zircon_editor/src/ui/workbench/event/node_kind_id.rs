use zircon_scene::NodeKind;

pub(super) fn node_kind_id(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Camera => "Camera",
        NodeKind::Cube => "Cube",
        NodeKind::Mesh => "Mesh",
        NodeKind::DirectionalLight => "DirectionalLight",
    }
}
