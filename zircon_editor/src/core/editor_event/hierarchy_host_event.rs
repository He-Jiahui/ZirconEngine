use serde::{Deserialize, Serialize};
use zircon_runtime::scene::NodeId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 宿主的层级编辑意图；真正的父子关系与命名约束由编辑状态应用 EditorIntent 时验证。
/// 成功派发后，执行层按状态变化通知呈现与反射视图。
pub enum EditorHierarchyEvent {
    ReparentNodes {
        node_ids: Vec<NodeId>,
        parent: Option<NodeId>,
    },
    RenameNode {
        node_id: NodeId,
        name: String,
    },
}
