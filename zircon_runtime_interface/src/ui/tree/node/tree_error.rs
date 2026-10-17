use thiserror::Error;

use crate::ui::event_ui::UiNodeId;

/// 基础树修改及 Runtime 树扩展返回的节点级错误；载荷保留出错节点或父节点标识。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiTreeError {
    /// 操作要求的节点不存在。
    #[error("ui tree is missing node {0:?}")]
    MissingNode(UiNodeId),
    /// 插入子节点时，指定父节点不存在。
    #[error("ui tree is missing parent {0:?}")]
    MissingParent(UiNodeId),
    /// 插入子节点的标识已被占用；重复根插入采用直接返回的另一条路径。
    #[error("ui tree already contains node {0:?}")]
    DuplicateNode(UiNodeId),
    /// Runtime 滚动扩展找到了节点，但该节点未携带滚动状态。
    #[error("ui tree node {0:?} is not scrollable")]
    NotScrollable(UiNodeId),
}
