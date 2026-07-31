use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::math::Transform;

use super::GizmoAxis;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportTransformPreview {
    pub node_id: NodeId,
    pub transform: Transform,
}

#[derive(Clone, Debug, Default)]
pub struct ViewportFeedback {
    pub hovered_axis: Option<GizmoAxis>,
    pub transformed_node: Option<NodeId>,
    pub(crate) transform_preview: Option<ViewportTransformPreview>,
    pub camera_updated: bool,
}
