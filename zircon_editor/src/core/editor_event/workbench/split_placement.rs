use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 新文档区相对于目标节点的位置；布局命令消费此值而非屏幕坐标。
pub enum SplitPlacement {
    Before,
    After,
}
