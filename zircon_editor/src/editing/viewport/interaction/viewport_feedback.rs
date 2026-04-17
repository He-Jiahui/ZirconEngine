use zircon_scene::NodeId;

use super::GizmoAxis;

#[derive(Clone, Debug, Default)]
pub struct ViewportFeedback {
    pub hovered_axis: Option<GizmoAxis>,
    pub transformed_node: Option<NodeId>,
    pub camera_updated: bool,
}
