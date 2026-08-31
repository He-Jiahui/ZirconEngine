use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::math::Transform;

use super::GizmoAxis;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportTransformRequest {
    pub primary: NodeId,
    pub target_pivot_world: Transform,
}

#[derive(Clone, Debug, Default)]
pub struct ViewportFeedback {
    pub hovered_axis: Option<GizmoAxis>,
    pub transformed_node: Option<NodeId>,
    pub(crate) transform_request: Option<ViewportTransformRequest>,
    pub camera_updated: bool,
    pub(crate) settings_changed: bool,
    pub(crate) interaction_extract_stale: bool,
}
