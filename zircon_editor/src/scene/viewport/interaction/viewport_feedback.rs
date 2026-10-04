//! 反馈把悬停、相机变化、产品失效和枢轴预览分开；目标变换尚未提交，只有宿主事务成功后才报告已变换实体。

use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::math::Transform;

use super::GizmoAxis;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 工具生成的目标枢轴请求；尚未写入世界，必须交给对应世界域的事务验证、预览和提交。
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
